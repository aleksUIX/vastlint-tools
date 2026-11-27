(() => {
  const entries = Array.from(document.querySelectorAll('.entry'));
  const search = document.querySelector('#query');
  if (!search) return;
  const count = document.querySelector('#matches');
  const searchable = entries.map(entry => entry.textContent.toLowerCase());
  function filter() {
    const query = search.value.trim().toLowerCase();
    let visible = 0;
    entries.forEach((entry, i) => {
      entry.hidden = query !== '' && !searchable[i].includes(query);
      if (!entry.hidden) visible++;
    });
    count.textContent = visible + ' of ' + entries.length + ' elements';
  }
  function reveal() {
    const target = document.getElementById(location.hash.slice(1));
    if (target && target.classList.contains('entry')) {
      if (target.hidden) { search.value = ''; filter(); }
      target.open = true;
      target.scrollIntoView({block:'start'});
    }
  }
  document.querySelector('.filter').hidden = false;
  search.addEventListener('input', filter);
  window.addEventListener('hashchange', reveal);
  document.addEventListener('click', event => {
    const link = event.target.closest('a');
    if (link && location.hash && link.hash === location.hash) reveal();
  });
  filter();
  reveal();
})();
