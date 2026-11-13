# Processador do Espia em nucleos (specs/020-codificar-pela-placa, contracts/medicao.md).
# So le, nao muda nada no computador. A cada intervalo soma o tempo de processador
# (KernelModeTime + UserModeTime, unidades de 100 ns) dos processos medidos; nucleos da leitura =
# variacao do tempo de processador / variacao do relogio. Entra na conta so o processo presente nas
# duas leituras seguidas (por ProcessId), entao processo que nasce ou morre no meio nao gera numero
# negativo nem salto.
#
#   powershell -File scripts/processador.ps1                       # arvore do Espia.exe, 60 s
#   powershell -File scripts/processador.ps1 -Nativo               # so o processo, sem filhos
#   powershell -File scripts/processador.ps1 -Processo encode-probe -Nativo -Duracao 30
param(
  # Nome do processo sem ".exe".
  [string]$Processo = 'Espia',
  # Mede so o processo, sem descer para os filhos.
  [switch]$Nativo,
  # Duracao da medicao, em segundos.
  [int]$Duracao = 60,
  # De quanto em quanto tempo le o processador, em segundos.
  [int]$Intervalo = 1
)

$Name = ($Processo -replace '\.exe$', '') + '.exe'
$Rotulo = $Name -replace '\.exe$', ''
if ($Intervalo -lt 1) { $Intervalo = 1 }

# Processos medidos agora: so os com esse nome (-Nativo) ou a arvore deles (como memoria.ps1).
function Get-Tree {
  $all = Get-CimInstance Win32_Process
  $roots = @($all | Where-Object { $_.Name -eq $Name })
  if (-not $roots) { return $null }
  if ($Nativo) { return $roots }
  $children = @{}
  foreach ($p in $all) {
    $key = [int]$p.ParentProcessId
    if (-not $children.ContainsKey($key)) { $children[$key] = New-Object System.Collections.ArrayList }
    [void]$children[$key].Add($p)
  }
  $out = New-Object System.Collections.ArrayList
  $queue = New-Object System.Collections.Queue
  foreach ($r in $roots) { $queue.Enqueue($r) }
  while ($queue.Count -gt 0) {
    $p = $queue.Dequeue()
    [void]$out.Add($p)
    $kids = $children[[int]$p.ProcessId]
    # Um PID reaproveitado pode ter "pai" mais novo que ele: so segue filhos criados depois do pai.
    if ($kids) { foreach ($k in $kids) { if ($k.CreationDate -ge $p.CreationDate) { $queue.Enqueue($k) } } }
  }
  return $out
}

# Tabela ProcessId -> tempo de processador (100 ns). Chave inclui a criacao, para PID reaproveitado.
function Read-Cpu {
  $tree = Get-Tree
  if (-not $tree) { return $null }
  $t = @{}
  foreach ($p in $tree) {
    $key = "{0}|{1}" -f [int]$p.ProcessId, $p.CreationDate.Ticks
    $t[$key] = [double]$p.KernelModeTime + [double]$p.UserModeTime
  }
  return $t
}

$logicos = [Environment]::ProcessorCount
$modo = if ($Nativo) { 'nativo' } else { 'arvore' }

$prev = Read-Cpu
if (-not $prev) {
  Write-Host "$Rotulo nao esta rodando ($Name nao encontrado)."
  exit 1
}

$clock = [System.Diagnostics.Stopwatch]::StartNew()
$prevAt = $clock.Elapsed.TotalSeconds
$vals = New-Object System.Collections.ArrayList

while ($clock.Elapsed.TotalSeconds -lt $Duracao) {
  Start-Sleep -Milliseconds ($Intervalo * 1000)
  $cur = Read-Cpu
  $at = $clock.Elapsed.TotalSeconds
  if (-not $cur) { break }   # o processo fechou: encerra com o que ja foi medido
  $dt = $at - $prevAt
  if ($dt -gt 0) {
    $delta = 0.0
    foreach ($k in $cur.Keys) {
      if ($prev.ContainsKey($k)) {
        $d = $cur[$k] - $prev[$k]
        if ($d -gt 0) { $delta += $d }
      }
    }
    # 100 ns -> segundos de processador; dividido pelos segundos de relogio = nucleos.
    [void]$vals.Add(($delta / 1e7) / $dt)
  }
  $prev = $cur
  $prevAt = $at
}

if ($vals.Count -eq 0) {
  Write-Host "$Rotulo nao esta rodando ($Name sumiu antes da primeira leitura)."
  exit 1
}

$media = ($vals | Measure-Object -Average).Average
$pico = ($vals | Measure-Object -Maximum).Maximum
$pct = $media / $logicos * 100
Write-Host ("{0} ({1}): {2:N2} nucleo medio, pico {3:N2} | {4:N1}% da maquina | {5} nucleos logicos | {6} amostras" -f $Rotulo, $modo, $media, $pico, $pct, $logicos, $vals.Count)
exit 0
