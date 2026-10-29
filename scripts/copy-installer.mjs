// Copia o instalador recém-gerado para public/download/Espia-setup.exe: o `npm run deploy` publica junto
// com o site, e o convite da tela inicial baixa sempre esse nome fixo (src/lib/app-invite.ts).
import { copyFileSync, mkdirSync, readFileSync } from 'node:fs';

const { version } = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
const from = `src-tauri/target/release/bundle/nsis/Espia_${version}_x64-setup.exe`;
mkdirSync('public/download', { recursive: true });
copyFileSync(from, 'public/download/Espia-setup.exe');
console.log(`instalador ${version} → public/download/Espia-setup.exe (publique com npm run deploy)`);
