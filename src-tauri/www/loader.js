// Confere se o Espia publicado responde e segue pra ele (levando a sala do link, se houver).
// Sem conexão, mostra a tela "Sem conexão" com "Tentar de novo". Ver contracts/deep-link.md.
const ORIGIN = 'https://espia.espia.workers.dev';
const ROOM_PATH = /^\/s\/[a-hjkmnp-z2-9]{3}-[a-hjkmnp-z2-9]{3}$/;
const TIMEOUT_MS = 8000;

const main = document.querySelector('main');
let path = new URLSearchParams(location.search).get('to');
if (!path || !ROOM_PATH.test(path)) path = '/';

async function reachable() {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), TIMEOUT_MS);
  try {
    // no-cors: só queremos saber se a rede chega lá; a resposta é opaca.
    await fetch(`${ORIGIN}/`, { method: 'HEAD', mode: 'no-cors', cache: 'no-store', signal: ctrl.signal });
    return true;
  } catch {
    return false;
  } finally {
    clearTimeout(timer);
  }
}

async function go() {
  main.dataset.state = 'loading';
  if (await reachable()) location.replace(ORIGIN + path);
  else main.dataset.state = 'offline';
}

// Link de sala chegando enquanto esta tela está aberta (src-tauri/src/lib.rs → open_room).
window.espiaOpen = (p) => {
  if (ROOM_PATH.test(p)) path = p;
  go();
};

document.getElementById('retry').addEventListener('click', go);
go();
