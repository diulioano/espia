import { mount } from 'svelte';
import '@fontsource/caprasimo/400.css';
import '@fontsource/figtree/400.css';
import '@fontsource/figtree/600.css';
import '@fontsource/figtree/700.css';
import './ui/tokens.css';
import './ui/base.css';
import App from './App.svelte';
import { installDesktopHook } from './lib/desktop';
import { router } from './lib/router.svelte';

installDesktopHook(router);

export default mount(App, { target: document.getElementById('app')! });
