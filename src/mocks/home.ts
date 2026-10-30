// Cenas mockadas da página inicial (D1).
import type { HomeView, SavedRoomsView, SavedRoomView } from '../lib/types';

export const HOME_SCENES = [
  { id: 'default', code: 'D1', label: 'Página inicial' },
  { id: 'empty', code: 'D1', label: 'Primeira visita (sem apelido)' },
  { id: 'error', code: 'D1', label: 'Link inválido' },
  { id: 'invite', code: 'D1', label: 'Convite do app (Windows)' },
  { id: 'invitehint', code: 'D1', label: 'Convite: baixando' },
  // Salas fixas (specs/011).
  { id: 'first', code: 'F1', label: 'Sem salas fixas · convite' },
  { id: 'create', code: 'F7', label: 'Modal Criar sala fixa' },
  { id: 'few', code: 'F2', label: 'Suas salas · 3 salas' },
  { id: 'menu', code: 'F3', label: 'Menu do item (dono)' },
  { id: 'many', code: 'F4', label: 'Suas salas · 5, agrupadas' },
  { id: 'loading', code: 'F5', label: 'Carregando quem está em cada sala' },
  { id: 'gone', code: 'F6', label: 'Sala que deixou de existir' },
  { id: 'mobile', code: 'M4', label: 'Celular com Suas salas', device: 'mobile' },
] as const;

export type HomeScene = (typeof HOME_SCENES)[number]['id'];

export const LINK_ERROR = 'Esse link não parece de uma sala.';

// Dados do Home.dc.html (specs/011).
const row = (code: string, name: string, over: Partial<SavedRoomView> = {}): SavedRoomView => ({
  code, name, own: false, tone: 'sage', state: 'ok', people: 0, streaming: 0, recent: 'usada ontem', ...over,
});
const FEW: SavedRoomView[] = [
  row('k7f-q2x', 'CS da galera', { people: 3, streaming: 1, own: true, tone: 'accent' }),
  row('m4p-t8a', 'Trampo do TCC', { people: 2, tone: 'sand' }),
  row('r2d-h7k', 'Filme de sexta', { tone: 'sage' }),
];

function roomsFor(scene: HomeScene): SavedRoomsView | null {
  const one = (rooms: SavedRoomView[]): SavedRoomsView => ({ groups: [{ label: null, rooms }], note: 'neste aparelho' });
  switch (scene) {
    case 'few':
    case 'mobile':
      return one(FEW.map((r) => ({ ...r })));
    case 'menu':
      return one(FEW.map((r, i) => ({ ...r, menuOpen: i === 0 })));
    case 'loading':
      return one(FEW.map((r) => ({ ...r, state: 'loading' })));
    case 'gone':
      return one([FEW[0], FEW[1], row('w9c-b3n', 'Minecraft do Caio', { state: 'gone', tone: 'deep' }), FEW[2]].map((r) => ({ ...r })));
    case 'many':
      return {
        groups: [
          {
            label: 'Com gente agora',
            rooms: [
              row('k7f-q2x', 'CS da galera', { people: 4, streaming: 2, own: true, tone: 'accent' }),
              row('h3k-q9w', 'Lu joga Hollow Knight', { people: 3, streaming: 1, tone: 'sage' }),
              row('m4p-t8a', 'Trampo do TCC', { people: 2, tone: 'sand' }),
            ],
          },
          {
            label: 'Usadas recentemente',
            rooms: [
              row('r2d-h7k', 'Filme de sexta', { tone: 'sage' }),
              row('c8v-n2m', 'Revisão de código', { recent: 'há 3 dias', own: true, tone: 'deep' }),
            ],
          },
        ],
        note: '5 salas neste aparelho',
      };
    default:
      return null;
  }
}

export function buildHomeScene(scene: HomeScene): HomeView {
  return {
    rooms: roomsFor(scene),
    // No celular o app não existe (specs/013).
    download: scene !== 'mobile',
    nickname: scene === 'empty' ? '' : 'Rafa',
    link: scene === 'error' ? 'https://youtu.be/dQw4w9WgXcQ' : '',
    linkError: scene === 'error' ? LINK_ERROR : null,
    appInvite: scene === 'invite' ? 'offer' : scene === 'invitehint' ? 'hint' : undefined,
    fixedDialog: scene === 'create' ? { mode: 'create', value: 'CS da galera', busy: false, error: null } : undefined,
  };
}
