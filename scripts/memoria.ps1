# Memória do Espia (specs/009-memoria-do-app, research R1).
# Soma a coluna "Memória" do Gerenciador de Tarefas (conjunto de trabalho privado) de toda a árvore do
# Espia.exe: o app + os processos do motor de página (WebView2) que ele abriu. O motor de outros apps
# (WhatsApp, Pesquisa do Windows) fica de fora.
#
#   powershell -File scripts/memoria.ps1           # total + detalhe por tipo
#   powershell -File scripts/memoria.ps1 -Watch    # uma linha a cada 5 s (Ctrl+C para sair)
param(
  [switch]$Watch,
  [int]$Interval = 5,
  [string]$Name = 'Espia.exe'
)

function Get-Tree {
  $all = Get-CimInstance Win32_Process
  $roots = @($all | Where-Object { $_.Name -eq $Name })
  if (-not $roots) { return $null }
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
    # Um PID reaproveitado pode ter "pai" mais novo que ele: só segue filhos criados depois do pai.
    if ($kids) { foreach ($k in $kids) { if ($k.CreationDate -ge $p.CreationDate) { $queue.Enqueue($k) } } }
  }
  return $out
}

function Get-Kind($p) {
  if ($p.Name -eq $Name) { return 'Espia.exe' }
  $cmd = [string]$p.CommandLine
  if ($cmd -match '--type=([\w-]+)') {
    switch ($Matches[1]) {
      'renderer' { return 'pagina (renderer)' }
      'gpu-process' { return 'GPU' }
      'utility' { return 'utilitarios' }
      'crashpad-handler' { return 'crashpad' }
      default { return $Matches[1] }
    }
  }
  if ($p.Name -eq 'msedgewebview2.exe') { return 'motor (principal)' }
  return $p.Name
}

function Measure-Espia {
  $tree = Get-Tree
  if (-not $tree) { return $null }
  $ids = @{}
  foreach ($p in $tree) { $ids[[int]$p.ProcessId] = $p }
  $perf = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process | Where-Object { $ids.ContainsKey([int]$_.IDProcess) }
  $mem = @{}
  foreach ($x in $perf) { $mem[[int]$x.IDProcess] = [double]$x.WorkingSetPrivate }
  $rows = foreach ($p in $tree) {
    [pscustomobject]@{
      Tipo = Get-Kind $p
      PID = [int]$p.ProcessId
      MB = [math]::Round(($mem[[int]$p.ProcessId]) / 1MB, 1)
      Motor = ($p.Name -eq 'msedgewebview2.exe')
    }
  }
  [pscustomobject]@{
    Rows = $rows
    Total = [math]::Round((($rows | Measure-Object MB -Sum).Sum), 1)
    Engine = @($rows | Where-Object Motor).Count
  }
}

if ($Watch) {
  Write-Host "Espia: memoria a cada $Interval s (Ctrl+C para sair)"
  while ($true) {
    $m = Measure-Espia
    $t = Get-Date -Format 'HH:mm:ss'
    if ($m) { Write-Host ("{0}  {1,7:N1} MB  ({2} processos do motor)" -f $t, $m.Total, $m.Engine) }
    else { Write-Host "$t  Espia nao esta rodando" }
    Start-Sleep -Seconds $Interval
  }
}

$m = Measure-Espia
if (-not $m) {
  Write-Host "Espia nao esta rodando ($Name nao encontrado)."
  exit 1
}
$m.Rows | Group-Object Tipo | ForEach-Object {
  [pscustomobject]@{
    Tipo = $_.Name
    Processos = $_.Count
    MB = [math]::Round((($_.Group | Measure-Object MB -Sum).Sum), 1)
  }
} | Sort-Object MB -Descending | Format-Table -AutoSize | Out-String | Write-Host
Write-Host ("Total: {0:N1} MB  ({1} processos do motor)" -f $m.Total, $m.Engine)
