# Contrato: endereço fixo do instalador (008)

## Endereço

```text
https://github.com/DiulianoVargas/espia/releases/latest/download/Espia-setup.exe
```

Responde com um redirecionamento para o arquivo `Espia-setup.exe` da Release mais nova (não rascunho, não pré-lançamento).

Página de reserva: `https://github.com/DiulianoVargas/espia/releases/latest`.

Ambos ficam em `src/lib/app-invite.ts` (`DOWNLOAD_URL`, `RELEASES_URL`) e em nenhum outro lugar do site.

## Obrigação do workflow de Release

Toda Release `v*` MUST ter, além de `Espia_<versão>_x64-setup.exe`, uma cópia idêntica chamada `Espia-setup.exe`. O `release.yml` sobe essa cópia depois do `tauri-action`:

```text
gh release upload <tag> Espia-setup.exe --clobber
```

Se a cópia faltar, o link do site cai num 404 do GitHub. Conferência: `curl -sIL <endereço> | grep -i "^location\|^HTTP"` deve terminar em `200`.
