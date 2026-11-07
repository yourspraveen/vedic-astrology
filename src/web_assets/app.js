/* Shared client script. Loads on every page; each block guards on the elements it needs. */
(function(){
  "use strict";
  var $ = function(id){ return document.getElementById(id); };

  /* ---------- theme ---------- */
  window.toggleTheme = function(){
    var d = document.documentElement, cur = d.getAttribute('data-theme');
    var sysDark = matchMedia('(prefers-color-scheme:dark)').matches;
    var dark = cur ? cur === 'dark' : sysDark;
    var now = dark ? 'light' : 'dark';
    d.setAttribute('data-theme', now);
    try { localStorage.setItem('fj-theme', now); } catch(e){}
    paintThemeBtn();
  };
  function paintThemeBtn(){
    var d = document.documentElement.getAttribute('data-theme');
    var dark = d ? d === 'dark' : matchMedia('(prefers-color-scheme:dark)').matches;
    var b = $('themebtn'); if (b){ b.textContent = dark ? '☀' : '☾'; }
  }

  /* ---------- localized names for glyph tooltips ---------- */
  var SIGN = {
    en:['Aries','Taurus','Gemini','Cancer','Leo','Virgo','Libra','Scorpio','Sagittarius','Capricorn','Aquarius','Pisces'],
    ta:['மேஷம்','ரிஷபம்','மிதுனம்','கடகம்','சிம்மம்','கன்னி','துலாம்','விருச்சிகம்','தனுசு','மகரம்','கும்பம்','மீனம்'],
    hi:['मेष','वृषभ','मिथुन','कर्क','सिंह','कन्या','तुला','वृश्चिक','धनु','मकर','कुम्भ','मीन']
  };
  function applyLang(l){
    var names = SIGN[l] || SIGN.en;
    document.querySelectorAll('.zodiac span[data-i]').forEach(function(s){
      var n = names[+s.getAttribute('data-i')] || '';
      s.title = n; s.setAttribute('aria-label', n);
    });
  }
  window.applyLang = applyLang;

  /* ---------- home: place lookup & options ---------- */
  function offsetLabel(h){
    var s = h<0?'-':'+', a = Math.abs(h), hh = Math.floor(a), mm = Math.round((a-hh)*60);
    return 'UTC'+s+String(hh).padStart(2,'0')+':'+String(mm).padStart(2,'0');
  }
  function setPlace(p){
    $('place').value = p.label; $('lat').value = (+p.lat).toFixed(4); $('lon').value = (+p.lon).toFixed(4);
    $('tz_name').value = p.tz_name || ''; $('zone').textContent = p.tz_name || '';
    if (p.tz !== null && p.tz !== undefined){ $('tz').value = p.tz; $('utc').textContent = offsetLabel(+p.tz); }
  }
  var found = [];
  window.fetchPlace = async function(){
    var f = $('f'); if(!f) return;
    $('zone').textContent = '…';
    var q = new URLSearchParams({ q:$('place').value, date:f.date.value, time:f.time.value });
    try{
      var r = await (await fetch('/api/geocode?'+q)).json();
      if (r.error || !r.places || !r.places.length){ $('zone').textContent = r.error || 'No place by that name was found.'; return; }
      found = r.places; setPlace(found[0]);
      var m = $('matches');
      m.replaceChildren.apply(m, found.map(function(p,i){
        var o = document.createElement('option'); o.value = i;
        o.textContent = p.label+' ('+(+p.lat).toFixed(2)+', '+(+p.lon).toFixed(2)+')';
        return o;
      }));
      m.style.display = found.length>1 ? '' : 'none';
    }catch(e){ $('zone').textContent = 'Lookup failed. Check the connection.'; }
  };
  window.openCalendar = function(){
    var f = $('f'); if(!f) return;
    var lang = (f.elements.lang && f.elements.lang.value) || 'en';
    var p = new URLSearchParams({ lat:$('lat').value, lon:$('lon').value, tz:$('tz').value, tz_name:$('tz_name').value, place:$('place').value, lang:lang });
    location.href = '/calendar?'+p;
  };

  /* ---------- loading overlay (blocks duplicate submits during slow AI enrichment) ---------- */
  var LOADMSG = {
    en: 'Generating your AI-enriched report… this can take a minute. Please don’t resubmit.',
    ta: 'உங்கள் AI-வளப்படுத்திய அறிக்கை உருவாக்கப்படுகிறது… ஒரு நிமிடம் ஆகலாம். மீண்டும் சமர்ப்பிக்க வேண்டாம்.',
    hi: 'आपकी AI-समृद्ध रिपोर्ट बन रही है… इसमें एक मिनट लग सकता है। कृपया दोबारा सबमिट न करें।'
  };
  var NETMSG = {
    en: 'The server could not be reached. Tap to close, then try again.',
    ta: 'சேவையகத்தை அணுக முடியவில்லை. மூடி, மீண்டும் முயலுங்கள்.',
    hi: 'सर्वर से संपर्क नहीं हो सका। बंद करें, फिर कोशिश करें।'
  };
  window.showLoading = function(msg){
    var lang = document.documentElement.lang || 'en';
    var o = $('fjload');
    if (!o){
      o = document.createElement('div'); o.id = 'fjload'; o.className = 'loadov';
      var c = document.createElement('div'); c.className = 'loadcard';
      var s = document.createElement('div'); s.className = 'spin';
      var p = document.createElement('p');
      var t = document.createElement('div'); t.className = 'ltimer';
      c.appendChild(s); c.appendChild(p); c.appendChild(t); o.appendChild(c); document.body.appendChild(o);
    }
    o.querySelector('p').textContent = msg || LOADMSG[lang] || LOADMSG.en;
    // Count up so the viewer sees it is working and how long it has been.
    var tmr = o.querySelector('.ltimer'), start = Date.now();
    var tick = function(){ var s = Math.floor((Date.now()-start)/1000); tmr.textContent = '⏱ ' + Math.floor(s/60) + ':' + ('0'+(s%60)).slice(-2); };
    tick();
    if (window._fjt) clearInterval(window._fjt);
    window._fjt = setInterval(tick, 1000);
    o.style.display = 'flex';
  };
  function stopTimer(){ if (window._fjt){ clearInterval(window._fjt); window._fjt = null; } }
  function loadError(form){
    var o = $('fjload'); stopTimer();
    if (!o) return;
    var sp = o.querySelector('.spin'); if (sp) sp.style.display = 'none';
    var tm = o.querySelector('.ltimer'); if (tm) tm.textContent = '';
    o.querySelector('p').textContent = NETMSG[document.documentElement.lang] || NETMSG.en;
    o.onclick = function(){ o.style.display = 'none'; o.onclick = null; };
    var b = form && form.querySelector('button[type=submit]'); if (b) b.disabled = false;
  }
  // Post a form and swap the reply in without ever leaving this page. A real navigation lets WebKit
  // (iOS Safari — and Chrome on iOS, which is also WebKit) suspend this page for the whole time the
  // POST is in flight: the spinner keeps turning because its rotation is a compositor-side CSS
  // animation, but setInterval stops firing, so the elapsed timer froze at 0:00. Staying put keeps
  // the timer ticking on every engine.
  function postPage(form){
    var action = form.getAttribute('action') || location.pathname;
    fetch(action, { method:'POST', body:new URLSearchParams(new FormData(form)), headers:{ Accept:'text/html' } })
      .then(function(r){ return r.text().then(function(t){ return { ct:r.headers.get('content-type')||'', txt:t }; }); })
      .then(function(res){
        stopTimer();
        var html = /text\/html/i.test(res.ct) ? res.txt
          : '<!DOCTYPE html><meta charset="utf-8"><body style="font:16px/1.6 system-ui;padding:24px">'
            + '<p>' + qaEsc(res.txt) + '</p><p><a href="/">\u2190 Back</a></p>';
        document.open(); document.write(html); document.close();
        try { history.replaceState({}, '', action); } catch(e){}   // keep the address bar as it was
      })
      .catch(function(){ loadError(form); });   // never auto-resubmit: a retry could re-bill the AI
  }
  // Called from the report form: show progress + lock the button only when the AI reading is on.
  window.onGenerate = function(){
    var f = $('f'); if (!f) return true;
    var ai = f.elements['ai'];
    if (!(ai && ai.checked)) return true;      // rule-based report is instant: let the browser post
    showLoading();
    var b = f.querySelector('button[type=submit]'); if (b) b.disabled = true;
    postPage(f);
    return false;                              // we post it ourselves, so cancel the navigation
  };

  /* ---------- site-wide language switch ---------- */
  window.switchLang = function(v){
    try { localStorage.setItem('fj-lang', v); } catch(e){}
    // On the report page, re-generate the report in the chosen language (the report is produced by
    // POST, so a plain ?lang= reload has nothing to render).
    var regen = $('regen');
    if (regen){ var rl = $('regen_lang'); if (rl) rl.value = v; showLoading('…'); postPage(regen); return; }
    var u = new URL(location.href); u.searchParams.set('lang', v); location.href = u.toString();
  };

  /* ---------- per-section "Rewrite with AI": submit the hidden rewrite form for that key ---------- */
  document.addEventListener('click', function(ev){
    var btn = ev.target && ev.target.closest ? ev.target.closest('.rewrite') : null;
    if (!btn) return;
    if (document.body.classList.contains('ai-off')) return; // no AI service: do nothing
    var form = $('rewritef'), sec = $('rewrite_section');
    if (!form || !sec) return;
    sec.value = btn.getAttribute('data-key') || '';
    showLoading('…');
    postPage(form);
  });

  /* ---------- open a saved .md report: read the file client-side, post its text ---------- */
  window.openSaved = function(ev){
    ev.preventDefault();
    var pick = $('openpick');
    var file = pick && pick.files && pick.files[0];
    if (!file){ return false; }
    var reader = new FileReader();
    reader.onload = function(){
      $('openfile').value = reader.result || '';
      $('openf').submit(); // submit() does not re-fire onsubmit, so no recursion
    };
    reader.readAsText(file);
    return false;
  };

  /* ---------- refresh UTC offset for the chosen date (daylight saving / history) ---------- */
  window.refreshOffset = async function(){
    var f = $('f'); if (!f) return;
    var zone = $('tz_name') ? $('tz_name').value : '';
    if (!zone) return; // a hand-typed offset — don't override it
    try {
      var p = new URLSearchParams({ tz_name:zone, date:f.date.value, time:f.time.value });
      var r = await (await fetch('/api/offset?'+p)).json();
      if (r.tz !== null && r.tz !== undefined){ $('tz').value = r.tz; $('utc').textContent = r.utc || offsetLabel(+r.tz); }
    } catch(e){}
  };

  /* ---------- readability: A+ / A- font sizing ---------- */
  window.bumpFont = function(dir){
    var z = 1; try { z = parseFloat(localStorage.getItem('fj-fontscale')) || 1; } catch(e){}
    z = Math.min(1.6, Math.max(0.8, Math.round((z + dir*0.1)*10)/10));
    document.documentElement.style.setProperty('--zoom', z);
    try { localStorage.setItem('fj-fontscale', z); } catch(e){}
  };

  /* ---------- calendar: jump to a month/year ---------- */
  window.goMonth = function(v){
    if (!v) return; var p = v.split('-'); if (p.length < 2) return;
    var u = new URL(location.href); u.searchParams.set('y', p[0]); u.searchParams.set('m', String(parseInt(p[1],10)));
    location.href = u.toString();
  };

  /* ---------- calendar: local timezone, geolocation, city picker, festival multiselect ---------- */
  function calNav(params, replace){
    var u = new URL(location.href);
    Object.keys(params).forEach(function(k){
      if (params[k] === null || params[k] === undefined) u.searchParams.delete(k);
      else u.searchParams.set(k, params[k]);
    });
    if (replace) location.replace(u.toString()); else location.href = u.toString();
  }
  function localTz(){ try { return Intl.DateTimeFormat().resolvedOptions().timeZone || ''; } catch(e){ return ''; } }
  // Every one of these used to be a silent `return`, so a refused permission, an unknown city name
  // or a slow lookup all looked identical to the button not working at all.
  var CALMSG = {
    en: { locating:'Finding your location…', insecure:'Browsers only share your location over HTTPS. Open this site with https://, or pick a city.',
          denied:'Location is turned off for this browser. Allow it in Settings \u203a Privacy \u203a Location Services, or pick a city.',
          locfail:'Could not get your location. Pick a city instead.', naming:'Finding the nearest city\u2026',
          here:'My location', typecity:'Type a city first.', looking:'Looking up…', none:'No place by that name was found.', neterr:'Lookup failed. Check the connection.' },
    ta: { locating:'உங்கள் இருப்பிடம் தேடப்படுகிறது…', insecure:'HTTPS வழியாக மட்டுமே உலாவி இருப்பிடத்தைப் பகிரும். https:// மூலம் திறக்கவும், அல்லது நகரத்தைத் தேர்வு செய்யவும்.',
          denied:'இந்த உலாவிக்கு இருப்பிடம் முடக்கப்பட்டுள்ளது. அமைப்புகளில் அனுமதிக்கவும், அல்லது நகரத்தைத் தேர்வு செய்யவும்.',
          locfail:'இருப்பிடத்தைப் பெற முடியவில்லை. நகரத்தைத் தேர்வு செய்யவும்.', naming:'அருகிலுள்ள நகரம் தேடப்படுகிறது\u2026',
          here:'எனது இருப்பிடம்', typecity:'முதலில் நகரத்தைத் தட்டச்சு செய்யவும்.', looking:'தேடப்படுகிறது…', none:'அந்தப் பெயரில் இடம் எதுவும் கிடைக்கவில்லை.', neterr:'தேடல் தோல்வி. இணைப்பைச் சரிபார்க்கவும்.' },
    hi: { locating:'आपका स्थान खोजा जा रहा है…', insecure:'ब्राउज़र स्थान केवल HTTPS पर साझा करते हैं। साइट को https:// से खोलें, या शहर चुनें.',
          denied:'इस ब्राउज़र के लिए स्थान बंद है। सेटिंग्स में अनुमति दें, या शहर चुनें.',
          locfail:'आपका स्थान नहीं मिल सका। इसके बजाय शहर चुनें.', naming:'निकटतम शहर खोजा जा रहा है\u2026',
          here:'मेरा स्थान', typecity:'पहले शहर लिखें.', looking:'खोज रहे हैं…', none:'उस नाम की कोई जगह नहीं मिली.', neterr:'खोज विफल। कनेक्शन जाँचें.' }
  };
  function calmsg(){ return CALMSG[document.documentElement.lang] || CALMSG.en; }
  function calSay(text, err){
    var el = $('calmsg'); if (!el) return;
    el.textContent = text || ''; el.className = 'cal-msg' + (err ? ' err' : '');
  }
  window.useMyLocation = function(){
    var m = calmsg();
    if (!navigator.geolocation){ calSay(m.locfail, true); return; }
    // iOS refuses geolocation outright on a plain-http origin and reports it as a plain denial,
    // which is indistinguishable from the user saying no — so name the real reason up front.
    if (window.isSecureContext === false){ calSay(m.insecure, true); return; }
    calSay(m.locating);                                   // acknowledge the tap straight away
    navigator.geolocation.getCurrentPosition(function(pos){
      var geo = { lat: pos.coords.latitude.toFixed(4), lon: pos.coords.longitude.toFixed(4), tz_name: localTz() };
      // Name the nearest city in the page's language, so the header says where it put you instead
      // of an anonymous "My location". Coordinates still drive every calculation; this is a label.
      calSay(m.naming);
      var lang = document.documentElement.lang || 'en';
      var done = function(place){
        geo.place = place;
        try { localStorage.setItem('fj-cal-geo', JSON.stringify(geo)); } catch(e){}
        // Drop any tz left over from the previous place: tz_name wins server-side, but if the
        // device has no resolvable zone the fallback must not be some other city's offset.
        calNav({ lat: geo.lat, lon: geo.lon, tz_name: geo.tz_name, place: place, tz: null });
      };
      fetch('/api/reverse?lat=' + geo.lat + '&lon=' + geo.lon + '&lang=' + encodeURIComponent(lang))
        .then(function(r){ return r.json(); })
        .then(function(j){ done((j && j.place) || m.here); })
        .catch(function(){ done(m.here); });      // no name available: fall back to the old label
    }, function(e){
      calSay(e && e.code === 1 ? m.denied : m.locfail, true);
    }, { enableHighAccuracy:false, timeout:15000, maximumAge:300000 });
  };
  // Also the submit handler of the little city form, so the iOS keyboard's Go key works — on iOS the
  // tap that dismisses the keyboard is swallowed, so the button alone is not a reliable way in.
  window.changeCity = function(ev){
    if (ev && ev.preventDefault) ev.preventDefault();
    var m = calmsg(), el = $('cityq'), q = ((el || {}).value || '').trim();
    if (!q){ calSay(m.typecity, true); if (el) el.focus(); return false; }
    if (el) el.blur();                                    // drop the iOS keyboard so the list shows
    calSay(m.looking);
    fetch('/api/geocode?q=' + encodeURIComponent(q)).then(function(r){ return r.json(); }).then(function(g){
      if (g && g.error){ calSay(g.error, true); return; }
      var p = g && g.places && g.places[0];
      if (!p){ calSay(m.none, true); return; }
      calNav({ lat: p.lat, lon: p.lon, tz_name: p.tz_name || '', place: p.label || q, tz: (p.tz != null ? p.tz : undefined) });
    }).catch(function(){ calSay(m.neterr, true); });
    return false;
  };
  function currentTrad(){
    return Array.prototype.slice.call(document.querySelectorAll('.tradbox:checked')).map(function(b){ return b.value; }).join(',');
  }
  window.applyTrad = function(){
    var trad = currentTrad();
    try { localStorage.setItem('fj-cal-trad', trad); } catch(e){}
    calNav({ trad: trad });
  };
  window.clearTrad = function(){
    try { localStorage.setItem('fj-cal-trad', ''); } catch(e){}
    calNav({ trad: '' });
  };
  // On the calendar page, restore saved traditions + geolocation when the URL doesn't specify them.
  (function initCalendar(){
    if (!document.querySelector('table.cal')) return;
    var u = new URL(location.href), changed = false;
    if (!u.searchParams.get('trad')) {
      var tr = ''; try { tr = localStorage.getItem('fj-cal-trad') || ''; } catch(e){}
      if (tr) { u.searchParams.set('trad', tr); changed = true; }
    }
    if (!u.searchParams.get('lat')) {
      var geo = null; try { geo = JSON.parse(localStorage.getItem('fj-cal-geo') || 'null'); } catch(e){}
      if (geo && geo.lat) {
        u.searchParams.set('lat', geo.lat); u.searchParams.set('lon', geo.lon);
        if (geo.tz_name) u.searchParams.set('tz_name', geo.tz_name);
        u.searchParams.set('place', geo.place || calmsg().here); changed = true;
      }
    }
    if (changed) location.replace(u.toString());
  })();

  /* ---------- birth details: export / import JSON ---------- */
  var BIRTH_KEYS = ['name','gender','date','time','place','lat','lon','tz','tz_name'];
  window.exportDetails = function(){
    var f = $('f'); if(!f) return;
    var data = {};
    BIRTH_KEYS.forEach(function(k){ var el = f.elements[k]; data[k] = el ? el.value : ''; });
    var blob = new Blob([JSON.stringify(data, null, 2)], { type:'application/json' });
    var a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    var who = (data.name||'birth').trim().replace(/[^\w.-]+/g,'_') || 'birth';
    var dob = (data.date||'').trim().replace(/[^\w-]+/g,'-');
    a.download = who + (dob ? '-'+dob : '') + '.json';
    document.body.appendChild(a); a.click();
    setTimeout(function(){ URL.revokeObjectURL(a.href); a.remove(); }, 0);
  };
  window.importDetails = function(file){
    if (!file) return; var f = $('f'); if(!f) return;
    var rd = new FileReader();
    rd.onload = function(){
      var d; try { d = JSON.parse(rd.result); } catch(e){ d = null; }
      if (!d || typeof d !== 'object'){ alert('That file is not valid birth-details JSON.'); return; }
      BIRTH_KEYS.forEach(function(k){ if (d[k] !== undefined && d[k] !== null && f.elements[k]) f.elements[k].value = d[k]; });
      var zone = $('zone'); if (zone) zone.textContent = d.tz_name || '';
      var utc = $('utc'); if (utc && d.tz !== undefined && d.tz !== '') utc.textContent = offsetLabel(+d.tz);
    };
    rd.readAsText(file);
  };

  /* ---------- report: ask a question + keep a Q&A log ---------- */
  var QA = [];
  function qaField(k){ var f = $('askf'); return (f && f.elements[k]) ? (f.elements[k].value || '') : ''; }
  function qaWho(){ return (qaField('name') || 'chart').trim().replace(/[^\w.-]+/g,'_') || 'chart'; }
  function qaDob(){ return (qaField('date') || '').trim().replace(/[^\w-]+/g,'-'); }
  function qaKey(){ var d = qaDob(); return 'fj-qa:' + qaWho() + (d ? '-'+d : ''); }
  function qaLoad(){ try { var v = localStorage.getItem(qaKey()); QA = v ? JSON.parse(v) : []; if (!Array.isArray(QA)) QA = []; } catch(e){ QA = []; } }
  function qaSave(){ try { localStorage.setItem(qaKey(), JSON.stringify(QA.slice(0,200))); } catch(e){} }
  function qaEsc(t){ var d = document.createElement('div'); d.textContent = (t==null?'':String(t)); return d.innerHTML; }
  function renderQa(){
    var log = $('qalog'), head = $('qahead'); if(!log) return;
    if (!QA.length){ log.innerHTML = ''; if (head) head.hidden = true; return; }
    if (head) head.hidden = false;
    log.innerHTML = QA.map(function(it){
      var when = it.t ? new Date(it.t).toLocaleString() : '';
      var src = (it.files && it.files.length) ? '<p class="qa-src">'+qaEsc('Sources: '+it.files.join(', '))+'</p>' : '';
      return '<div class="qa-item"><p class="qa-q">'+qaEsc(it.q)+'</p><div class="qa-a">'+qaEsc(it.a)+'</div>'+src+'<p class="qa-t">'+qaEsc(when)+'</p></div>';
    }).join('');
  }
  window.qaInit = function(){ if ($('askf')){ qaLoad(); renderQa(); } };
  window.ask = function(ev){
    if (ev) ev.preventDefault();
    var a = $('answer'), f = $('askf'); if(!a||!f) return false;
    var qel = $('question'), q = qel ? qel.value.trim() : '';
    if (!q) return false;
    a.hidden = false; a.className = 'answer'; a.textContent = '…';
    var body = new URLSearchParams(new FormData(f));
    fetch('/api/ask', { method:'POST', body:body }).then(function(r){ return r.json(); }).then(function(r){
      if (r.error){ a.textContent = r.error; return; }
      a.hidden = true;                       // the log below is the lasting record
      QA.unshift({ q:q, a:r.answer, files:r.files||[], t:Date.now() });
      qaSave(); renderQa();
      if (qel) qel.value = '';
    }).catch(function(){ a.textContent = 'Could not reach the server.'; });
    return false;
  };
  window.exportQa = function(){
    if (!QA.length) return;
    var name = qaField('name'), dob = qaField('date'), place = qaField('place');
    var out = ['# Questions & answers — ' + (name || 'chart'), ''];
    if (dob || place) out.push('Born ' + [dob, place].filter(Boolean).join(' · '));
    out.push('Exported ' + new Date().toLocaleString() + ' · Vedic Astrology', '');
    QA.slice().reverse().forEach(function(it){      // oldest first in the file
      out.push('## ' + (it.t ? new Date(it.t).toLocaleString() : ''));
      out.push('**Q:** ' + it.q, '', it.a, '');
      if (it.files && it.files.length) out.push('_Sources: ' + it.files.join(', ') + '_', '');
      out.push('---', '');
    });
    var blob = new Blob([out.join('\n')], { type:'text/markdown' });
    var a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    var d = qaDob();
    a.download = qaWho() + (d ? '-'+d : '') + '-qa.md';
    document.body.appendChild(a); a.click();
    setTimeout(function(){ URL.revokeObjectURL(a.href); a.remove(); }, 0);
  };

  /* ---------- settings: test, load models, save ---------- */
  window.aiTest = async function(){
    var m = $('testmsg'); if(!m) return; m.className=''; m.textContent='Testing…';
    try{ var r = await (await fetch('/api/ai/test',{method:'POST'})).json();
      if (r.ok){ m.className='ok'; m.textContent='✓ '+r.ok; } else { m.className='err'; m.textContent='✕ '+(r.error||'failed'); }
    }catch(e){ m.className='err'; m.textContent='✕ request failed'; }
  };
  window.aiLoadModels = async function(){
    var m = $('testmsg'); if(m){ m.className=''; m.textContent='Loading models…'; }
    try{
      var r = await (await fetch('/api/ai/models')).json();
      if (r.error){ if(m){ m.className='err'; m.textContent='✕ '+r.error; } return; }
      // Fill both the report (fast) and Ask (reasoning) pickers from the same list, each keeping its own choice.
      ['model','ask_model'].forEach(function(id){
        var sel = $(id); if(!sel) return;
        var cur = sel.value || (id==='model' ? r.current : '') || '';
        var ids = r.models.slice();
        if (cur && ids.indexOf(cur)<0){ ids.unshift(cur); }
        sel.replaceChildren.apply(sel, ids.map(function(mid){
          var o = document.createElement('option'); o.textContent = mid; o.selected = (mid===cur); return o;
        }));
      });
      if(m){ m.className='ok'; m.textContent='✓ '+r.models.length+' model(s) loaded.'; }
    }catch(e){ if(m){ m.className='err'; m.textContent='✕ could not load models'; } }
  };
  window.aiSave = async function(ev){
    if (ev) ev.preventDefault();
    var f = $('setf'), s = $('savedmsg'); if(!f) return false;
    var body = new URLSearchParams(new FormData(f));
    try{ var r = await (await fetch('/api/settings',{method:'POST',body:body})).json();
      if (s){ s.textContent = r.ok ? 'Saved ✓' : (r.error||'failed'); }
    }catch(e){ if(s){ s.textContent='Save failed'; } }
    return false;
  };
  // theme control inside settings
  window.setTheme = function(v){
    var d = document.documentElement;
    if (v==='system'){ d.removeAttribute('data-theme'); try{ localStorage.removeItem('fj-theme'); }catch(e){} }
    else { d.setAttribute('data-theme', v); try{ localStorage.setItem('fj-theme', v); }catch(e){} }
    paintThemeBtn();
  };

  /* ---------- wire up on load ---------- */
  document.addEventListener('DOMContentLoaded', function(){
    paintThemeBtn();
    if (window.qaInit) window.qaInit();
    var f = $('f');
    if (f){
      var preset = $('preset');
      if (preset) preset.onchange = function(e){ var o=e.target.selectedOptions[0]; if(!o.dataset.lat) return;
        setPlace({label:o.textContent,lat:o.dataset.lat,lon:o.dataset.lon,tz_name:o.dataset.zone,tz:+o.dataset.tz}); };
      var m = $('matches'); if (m) m.onchange = function(e){ setPlace(found[e.target.value]); };
      var tz = $('tz'); if (tz) tz.oninput = function(){ $('tz_name').value=''; $('zone').textContent=''; $('utc').textContent=offsetLabel(+tz.value); };
      var lang = f.lang; if (lang){ applyLang(lang.value); lang.onchange = function(){ applyLang(lang.value); }; }
    } else {
      applyLang('en');
    }
    // Settings: reflect the stored theme in the radio group.
    var radios = document.querySelectorAll('input[name=theme]');
    if (radios.length){
      var stored = 'system';
      try { stored = localStorage.getItem('fj-theme') || 'system'; } catch(e){}
      radios.forEach(function(r){ if (r.value === stored) r.checked = true; });
    }
  });
})();
