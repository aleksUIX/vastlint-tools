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

  function filter() {
    const q = search.value.trim().toLowerCase();
    let visible = 0;
    for (const entry of entries) {
      let show = !scope.value || entry.dataset.purpose === scope.value;
      if (show && q) {
        if (!searchText.has(entry)) searchText.set(entry, entry.textContent.toLowerCase());
        show = searchText.get(entry).includes(q);
      }
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
    for (const item of panels) item.hidden = item !== panel;
    for (const tab of tabs) {
      if (tab.hash === '#' + panel.id) tab.setAttribute('aria-current', 'page');
      else tab.removeAttribute('aria-current');
    }
    if (!target) return;
    if (target.classList.contains('entry')) {
      if (target.hidden) clearFilters();
      target.open = true;
    }
    if (scroll) {
      const destination = target === panel ? navigation : target;
      destination.scrollIntoView({block: 'start'});
    }
  }

  if (search) {
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
