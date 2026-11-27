(() => {
  const navigation = document.querySelector('.views');
  const panels = Array.from(document.querySelectorAll('.panel'));
  const tabs = Array.from(navigation.querySelectorAll('a'));
  const entries = Array.from(document.querySelectorAll('.entry'));
  const search = document.querySelector('#query');
  const scope = document.querySelector('#scope');
  const matches = document.querySelector('#matches');
  const empty = document.querySelector('#empty');
  const searchText = new WeakMap();
  const assets = new Map();

  // Resolve each URL from the declaring element, never from a tracking or click field.
  for (const card of document.querySelectorAll('.asset')) {
    const button = card.querySelector('button');
    if (!button) continue;
    const entry = card.closest('.entry');
    const kind = card.dataset.kind;
    const embedded = card.hasAttribute('data-file') || card.hasAttribute('data-inline');
    const declared = entry.querySelector('.value')?.textContent.trim();
    if (!declared) continue;
    let source = declared;
    if (card.dataset.file) {
      const file = document.getElementById(card.dataset.file);
      source = file.localName === 'image' ? file.getAttribute('href') :
        'data:' + (card.dataset.mime || 'application/octet-stream') + ';base64,' + file.content.textContent;
    }
    assets.set(entry.id, {kind, source, embedded, filename: card.querySelector('strong').textContent});
    let frame;
    let media;
    let message;

    function createFrame() {
      frame = document.createElement('figure');
      frame.className = 'asset-preview';
      frame.hidden = true;
      message = document.createElement('figcaption');
      message.setAttribute('role', 'status');
      frame.append(message);
      entry.querySelector('.inside').prepend(frame);
    }

    function stop() {
      if (!media) return;
      media.removeAttribute('src');
      if (kind !== 'image') media.load();
      media.remove();
      media = null;
      frame.hidden = true;
      button.textContent = (embedded ? 'View ' : 'Load ') + kind;
      button.setAttribute('aria-expanded', 'false');
    }

    function thumbnail(source) {
      if (kind === 'video') {
        const video = document.createElement('video');
        video.muted = true;
        video.playsInline = true;
        video.preload = 'metadata';
        video.setAttribute('aria-hidden', 'true');
        video.addEventListener('loadedmetadata', () => {
          if (video.duration > 0) video.currentTime = Math.min(0.05, video.duration / 2);
        });
        video.addEventListener('loadeddata', () => card.querySelector('.asset-thumb').replaceChildren(video));
        video.src = source;
        return;
      }
      const img = new Image();
      img.alt = '';
      img.referrerPolicy = 'no-referrer';
      img.onload = () => card.querySelector('.asset-thumb').replaceChildren(img);
      img.src = source;
    }

    function show() {
      if (!frame) createFrame();
      media = document.createElement(kind === 'image' ? 'img' : kind);
      const current = media;
      const filename = card.querySelector('strong').textContent;
      if (kind === 'image') {
        media.alt = filename;
        media.referrerPolicy = 'no-referrer';
      } else {
        media.controls = true;
        media.preload = 'metadata';
        media.setAttribute('aria-label', filename);
        if (kind === 'video') media.playsInline = true;
      }
      frame.prepend(media);
      frame.hidden = false;
      entry.open = true;
      button.textContent = 'Hide ' + kind;
      button.setAttribute('aria-expanded', 'true');
      message.textContent = 'Loading ' + filename + '…';
      media.addEventListener(kind === 'image' ? 'load' : 'loadedmetadata', () => {
        if (media !== current) return;
        message.textContent = filename + (embedded ? ' · embedded' : ' · remote media');
      });
      media.addEventListener('error', () => {
        if (media !== current) return;
        stop();
        frame.hidden = false;
        message.textContent = 'Could not display ' + filename + '. Check the URL, file availability and browser format support.';
      });
      media.src = source;
    }

    button.hidden = false;
    button.setAttribute('aria-expanded', 'false');
    if (embedded && kind !== 'audio') thumbnail(source);
    button.addEventListener('click', event => {
      event.preventDefault();
      if (media) stop();
      else show();
    });
    entry.addEventListener('toggle', () => { if (!entry.open) stop(); });
    // Stop playback when navigating or filtering the declaration out of view.
    entry.addEventListener('previewhide', stop);
  }

  async function copyText(text, status, content) {
    try {
      await navigator.clipboard.writeText(text);
      status.textContent = 'Text copied';
    } catch {
      if (content) {
        const range = document.createRange();
        range.selectNodeContents(content);
        const selection = window.getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
        status.textContent = 'Text selected. Use your copy command.';
      } else {
        const field = document.createElement('textarea');
        field.readOnly = true;
        field.value = text;
        field.setAttribute('aria-label', 'QA notes to copy');
        status.replaceChildren('Select and copy these notes.', field);
        field.select();
      }
    }
  }

  const studio = document.getElementById('qa-studio');
  if (studio) {
    const payload = document.querySelector('#source pre code').textContent;
    const bytes = new TextEncoder().encode(JSON.stringify({sourceMode: 'xml', action: 'validate', payload}));
    // Leave larger documents on the copy/paste path to keep the session URL portable.
    if (bytes.length <= 48 * 1024) {
      const encoded = btoa(Array.from(bytes, b => String.fromCharCode(b)).join(''));
      studio.hash = 'session=' + encoded.replaceAll('+','-').replaceAll('/','_').replace(/=+$/,'');
      studio.textContent = 'Open full tag in SIMID Studio';
      document.getElementById('studio-help').hidden = true;
    }
  }
  const stopPlayer = setupQAPlayer(assets, copyText);
  const wrap = document.getElementById('wrap-lines');
  if (wrap) {
    document.getElementById('wrap-control').hidden = false;
    wrap.addEventListener('change', () => document.getElementById('entries').classList.toggle('wrapped', wrap.checked));
  }
  const copyXml = document.getElementById('copy-xml');
  copyXml.hidden = false;
  copyXml.addEventListener('click', () => {
    const code = document.querySelector('#source pre code');
    copyText(code.textContent, copyXml.nextElementSibling, code);
  });

  for (const button of document.querySelectorAll('.copy-handoff')) {
    button.hidden = false;
    button.addEventListener('click', () => {
      const brief = button.closest('.ad').querySelector('.brief');
      copyText(brief.innerText, button.nextElementSibling, brief);
    });
  }

  function filter() {
    const q = search.value.trim().toLowerCase();
    let visible = 0;
    for (const entry of entries) {
      let show = scope.value === 'findings' ? Boolean(entry.querySelector('.issue-count')) :
        !scope.value || entry.dataset.purpose === scope.value;
      if (show && q) {
        if (!searchText.has(entry)) searchText.set(entry, entry.textContent.toLowerCase());
        show = searchText.get(entry).includes(q);
      }
      if (!show && !entry.hidden && entry.querySelector('.asset-preview')) entry.dispatchEvent(new Event('previewhide'));
      entry.hidden = !show;
      if (show) visible++;
    }
    matches.textContent = q || scope.value ? visible + ' of ' + entries.length : '';
    empty.hidden = visible !== 0;
  }

  function clearFilters() {
    search.value = '';
    scope.value = '';
    filter();
  }

  function showView(scroll) {
    const target = document.getElementById(location.hash.slice(1));
    const panel = target?.closest('.panel') || document.getElementById(navigation.dataset.first);
    if (panel.id !== 'player') stopPlayer();
    if (panel.id !== 'elements') {
      for (const frame of document.querySelectorAll('.asset-preview:not([hidden])')) {
        frame.closest('.entry').dispatchEvent(new Event('previewhide'));
      }
    }
    for (const item of panels) item.hidden = item !== panel;
    for (const tab of tabs) {
      if (tab.hash === '#' + panel.id) tab.setAttribute('aria-current', 'page');
      else tab.removeAttribute('aria-current');
    }
    if (!target) return;
    const playerOption = target.matches('#qa-asset option');
    if (playerOption && !target.selected) {
      target.selected = true;
      target.closest('select').dispatchEvent(new Event('change'));
    }
    if (target.classList.contains('entry')) {
      if (target.hidden) clearFilters();
      target.open = true;
    }
    if (scroll) {
      const destination = target === panel || playerOption ? navigation : target;
      destination.scrollIntoView({block: 'start'});
    }
  }

  if (search) {
    document.getElementById('entries').addEventListener('keydown', event => {
      if (!event.target.matches('.entry > summary') || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
      if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
      const code = event.target.querySelector('.snippet');
      if (code.scrollWidth <= code.clientWidth) return;
      event.preventDefault();
      code.scrollLeft += event.key === 'ArrowRight' ? 64 : -64;
    });
    document.querySelector('.tools').hidden = false;
    search.addEventListener('input', filter);
    scope.addEventListener('change', filter);
    document.querySelector('#clear').addEventListener('click', () => {
      clearFilters();
      search.focus();
    });
  }
  document.addEventListener('click', event => {
    const link = event.target.closest('a[href^="#"]');
    if (link && location.hash === link.hash) showView(true);
  });
  window.addEventListener('hashchange', () => showView(true));
  showView(Boolean(location.hash));
})();
