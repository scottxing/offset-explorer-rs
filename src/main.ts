// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License

import { mount } from 'svelte';
import App from './App.svelte';

const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;
