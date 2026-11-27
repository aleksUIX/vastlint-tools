function setupQAPlayer(assets, copyText) {
  const byId = id => document.getElementById('qa-' + id);
  const select = byId('asset');
  if (!select) return () => {};
  for (const option of Array.from(select.options)) {
    const asset = assets.get(option.value);
    if (!asset) option.remove();
    else {
      option.textContent += ' · ' + asset.filename;
      option.id = 'preview-' + option.value;
      const link = document.createElement('a');
      link.href = '#' + option.id;
      link.textContent = 'Open in QA player';
      document.getElementById(option.value).querySelector('.references').append(link);
    }
  }
  for (const link of document.querySelectorAll('#trafficking .resources li > div > a:first-child')) {
    const target = document.getElementById('preview-' + link.hash.slice(1));
    if (!target) continue;
    const preview = document.createElement('a');
    preview.href = '#' + target.id;
    preview.textContent = 'Preview';
    link.parentElement.append(' · ', preview);
  }
  for (const group of Array.from(select.children)) {
    if (!group.children.length) group.remove();
  }
  if (!select.options.length) {
    byId('empty').textContent = 'This response has no directly previewable media.';
    return () => {};
  }
  byId('empty').hidden = true;
  byId('controls').hidden = false;

  const trackers = new Map();
  for (const link of byId('tracking').children) {
    const key = link.dataset.owner + '/' + link.dataset.event;
    if (!trackers.has(key)) trackers.set(key, []);
    trackers.get(key).push(link.hash);
  }
  let media;
  let measuredRatio;
  const option = () => select.selectedOptions[0];
  const asset = () => assets.get(select.value);
  const rows = byId('events');
  const stage = byId('stage');
  const result = byId('result');
  const load = byId('load');
  function resize() {
    const ratio = byId('ratio').value;
    const {width,height} = option().dataset;
    const declared = Number(width) > 0 && Number(height) > 0 ? Number(width) / Number(height) : null;
    const value = ratio === 'asset' ? (measuredRatio || declared || 16/9) :
      ratio.split(' / ').map(Number).reduce((a,b) => a/b);
    stage.style.aspectRatio = value;
    stage.style.setProperty('--stage-ratio', value);
  }
  function note(message, event) {
    const li = document.createElement('li');
    const time = media && Number.isFinite(media.currentTime) ? media.currentTime : 0;
    li.textContent = time.toFixed(1) + ' s · ' + message;
    if (event) {
      const {owner, shared} = option().dataset;
      const refs = [...(trackers.get(owner + '/' + event) || []), ...(trackers.get(shared + '/' + event) || [])];
      for (const [i, hash] of refs.entries()) {
        const link = document.createElement('a');
        link.href = hash;
        link.textContent = 'Tracker ' + (i + 1);
        li.append(' ', link);
      }
    }
    if (rows.children.length >= 150) rows.firstElementChild.remove();
    rows.append(li);
    rows.scrollTop = rows.scrollHeight;
  }
  function stop() {
    if (!media) return;
    const previous = media;
    media = null;
    previous.remove();
    previous.removeAttribute('src');
    if (previous.localName !== 'img') previous.load();
    byId('placeholder').hidden = false;
    byId('placeholder').textContent = 'Preview stopped.';
    load.textContent = asset().embedded ? 'Load preview' : 'Load remote preview';
  }
  function selectAsset() {
    stop();
    measuredRatio = null;
    const {width,height,slot,duration} = option().dataset;
    const facts = [width && height && width + ' × ' + height, slot && 'Slot ' + slot, duration].filter(Boolean);
    byId('declaration').href = '#' + select.value;
    byId('declared').textContent = facts.join(' · ') || 'Dimensions and duration unspecified';
    byId('observed').textContent = 'Awaiting preview';
    byId('placeholder').textContent = 'Load the asset to preview it here.';
    load.textContent = asset().embedded ? 'Load preview' : 'Load remote preview';
    result.textContent = asset().embedded ? 'Embedded in this report.' : 'Loads media from the declared host.';
    rows.replaceChildren();
    byId('copy-status').replaceChildren();
    resize();
  }
  function inspect(current) {
    const w = current.videoWidth || current.naturalWidth;
    const h = current.videoHeight || current.naturalHeight;
    const duration = current.duration;
    const facts = [], differences = [];
    const data = option().dataset;
    if (w && h) {
      measuredRatio = w/h;
      facts.push(w + ' × ' + h);
      if (Number(data.width) > 0 && Number(data.height) > 0 && (Number(data.width) !== w || Number(data.height) !== h)) differences.push('File dimensions differ from VAST.');
    }
    if (Number.isFinite(duration)) {
      facts.push(duration.toFixed(2) + ' s');
      if (/^\d+:\d{2}:\d{2}(\.\d+)?$/.test(data.duration)) {
        const [h,m,s] = data.duration.split(':').map(Number);
        const delta = duration - (h * 3600 + m * 60 + s);
        if (Math.abs(delta) >= 0.1) differences.push('Duration differs by ' + Math.abs(delta).toFixed(2) + ' s.');
      }
    }
    byId('observed').textContent = facts.join(' · ') || 'Browser supplied no measurements';
    result.textContent = differences.join(' ') || 'Media loaded.';
    resize();
    note('Media loaded');
  }
  load.addEventListener('click', () => {
    if (media) {
      note('Preview stopped');
      stop();
      return;
    }
    const {kind,source,filename} = asset();
    const current = document.createElement(kind === 'image' ? 'img' : kind);
    media = current;
    if (kind === 'image') {
      current.alt = filename;
      current.referrerPolicy = 'no-referrer';
    } else {
      current.controls = true;
      current.preload = 'metadata';
      current.playsInline = true;
      current.setAttribute('aria-label', filename);
    }
    byId('media').replaceChildren(current);
    byId('placeholder').hidden = true;
    load.textContent = 'Stop preview';
    byId('observed').textContent = 'Loading';
    result.textContent = 'Loading media…';
    current.addEventListener(kind === 'image' ? 'load' : 'loadedmetadata', () => {
      if (current === media) inspect(current);
    });
    current.addEventListener('error', () => {
      if (current !== media) return;
      note('Load failed');
      stop();
      byId('observed').textContent = 'Unavailable';
      byId('placeholder').textContent = 'Media unavailable';
      result.textContent = 'Check the file URL, network, and browser format support.';
    });
    if (kind !== 'image') {
      let started = false;
      const milestones = new Set();
      let previousTime = 0;
      const recordMilestones = () => {
        if (media !== current || !Number.isFinite(current.duration) || !started || current.seeking) return;
        for (const [fraction,event] of [[0.25,'firstQuartile'],[0.5,'midpoint'],[0.75,'thirdQuartile']]) {
          if (previousTime < current.duration * fraction && current.currentTime >= current.duration * fraction && !milestones.has(event)) {
            milestones.add(event);
            note(event, event);
          }
        }
        previousTime = current.currentTime;
      };
      current.addEventListener('seeking', () => { previousTime = current.currentTime; });
      current.addEventListener('playing', () => {
        if (media !== current) return;
        note(started ? 'Resumed' : 'Playback started', started ? 'resume' : 'start');
        started = true;
      });
      current.addEventListener('pause', () => {
        if (media === current && !current.ended) note('Paused','pause');
      });
      current.addEventListener('seeked', () => {
        if (media === current) note('Seeked to ' + current.currentTime.toFixed(1) + ' s');
      });
      current.addEventListener('timeupdate', recordMilestones);
      current.addEventListener('ended', () => {
        if (media !== current) return;
        recordMilestones();
        note('Playback complete','complete');
        previousTime = 0;
        milestones.clear();
        started = false;
      });
    }
    current.src = source;
  });
  byId('ratio').addEventListener('change',resize);
  byId('guides').addEventListener('change',e=>{byId('guide').hidden = !e.target.checked;});
  select.addEventListener('change',selectAsset);
  byId('copy').addEventListener('click',() => {
    const text = [document.querySelector('.filename').textContent, option().parentElement.label,
      option().textContent, 'Declared: ' + byId('declared').textContent, 'Observed: ' + byId('observed').textContent,
      result.textContent, ...Array.from(rows.children, r=>r.textContent)].join('\n');
    copyText(text, byId('copy-status'));
  });
  selectAsset();
  return stop;
}
