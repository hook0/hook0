// Animated logo wall (homepage social proof).
// Loaded as a Parcel-bundled module from _customers.ejs (homepage only).
// Every tick, all grid slots crossfade at once to a freshly shuffled arrangement.
// The pool is larger than the grid, so logos rotate in/out over time. A "pinned"
// logo (Coinbase) is always part of the arrangement and always moves to a new slot,
// so it keeps animating yet stays continuously visible on screen.
document.addEventListener('DOMContentLoaded', function() {
    var rotator = document.querySelector('[data-logo-rotator]');
    if (!rotator) return; // only present on the homepage

    var poolEl = document.querySelector('[data-logo-pool]');
    if (!poolEl) return;

    // Pool entries come from the bundler-resolved <img> tags: `logo` is the stable
    // identity (original path, also on each grid cell), `src` is the final hashed URL.
    var pool = Array.prototype.slice.call(poolEl.querySelectorAll('img')).map(function(img) {
        return {
            logo: img.getAttribute('data-logo'),
            src: img.getAttribute('src'),
            name: img.getAttribute('data-name') || '',
            pinned: img.getAttribute('data-pinned') === 'true',
        };
    });
    if (pool.length === 0) return;

    var byLogo = {};
    pool.forEach(function(c) { byLogo[c.logo] = c; });

    var cells = Array.prototype.slice.call(rotator.querySelectorAll('[data-logo-cell]'));
    // Need at least one more logo than slots, otherwise there is nothing to rotate in.
    // This gentle opacity crossfade runs even under prefers-reduced-motion by design:
    // it has no movement/translation, only a soft fade (CSS re-asserts the transition
    // inside the reduced-motion block so it stays smooth rather than a hard cut).
    if (cells.length === 0 || pool.length <= cells.length) return;

    // Per-slot state: current logo + which <img> layer is active.
    var slots = cells.map(function(cell) {
        var active = cell.querySelector('.logo-layer--active') || cell.querySelector('.logo-layer');
        return {
            layers: cell.querySelectorAll('.logo-layer'),
            active: active,
            logo: active.getAttribute('data-logo'),
        };
    });

    // Pinned logos are always shown; the rest cycle through a rolling window. With more
    // logos than slots, some sit off-grid each tick and rotate in over time.
    var pinnedLogos = pool.filter(function(c) { return c.pinned; }).map(function(c) { return c.logo; });
    var cycleLogos = pool.filter(function(c) { return !c.pinned; }).map(function(c) { return c.logo; });
    var nonPinnedSlots = slots.length - pinnedLogos.length;
    if (nonPinnedSlots < 0 || cycleLogos.length < nonPinnedSlots) return; // can't fill the grid
    var cursor = 1; // advance the rolling window so the first tick already brings variety

    // Crossfade a slot to a new logo: fill the inactive layer, then swap which layer
    // is active so the opacity transitions overlap (true crossfade, no blank frame).
    function setSlotLogo(slot, logo) {
        if (slot.logo === logo) return;
        var meta = byLogo[logo] || { name: '', logo: logo };
        var incoming = null;
        for (var i = 0; i < slot.layers.length; i++) {
            if (slot.layers[i] !== slot.active) { incoming = slot.layers[i]; break; }
        }
        if (!incoming) return;
        incoming.setAttribute('src', meta.src || meta.logo);
        incoming.setAttribute('alt', meta.name);
        incoming.setAttribute('data-logo', meta.logo);
        void incoming.offsetWidth; // commit opacity:0 before transitioning in
        incoming.classList.add('logo-layer--active');
        slot.active.classList.remove('logo-layer--active');
        slot.active.setAttribute('alt', ''); // only the visible layer keeps a meaningful alt
        slot.active = incoming;
        slot.logo = logo;
    }

    function randomInt(n) { return Math.floor(Math.random() * n); }

    // Assign `logos` to slots so no slot keeps its current logo — so every slot visibly
    // crossfades. Each logo is forbidden from at most one slot (where it sits now), so a
    // valid arrangement always exists: shuffle-and-check, with a deterministic repair pass.
    function arrangeWithoutFixedPoints(logos, current) {
        for (var attempt = 0; attempt < 50; attempt++) {
            var a = logos.slice();
            for (var i = a.length - 1; i > 0; i--) {
                var j = randomInt(i + 1);
                var t = a[i]; a[i] = a[j]; a[j] = t;
            }
            var ok = true;
            for (var s = 0; s < a.length; s++) { if (a[s] === current[s]) { ok = false; break; } }
            if (ok) return a;
        }
        var res = logos.slice();
        for (var s2 = 0; s2 < res.length; s2++) {
            if (res[s2] === current[s2]) {
                for (var k = 0; k < res.length; k++) {
                    if (k !== s2 && res[k] !== current[s2] && res[s2] !== current[k]) {
                        var tmp = res[s2]; res[s2] = res[k]; res[k] = tmp; break;
                    }
                }
            }
        }
        return res;
    }

    function tick() {
        // Rolling window of non-pinned logos (wraps around the cycle) + the pinned logo(s).
        var visible = [];
        for (var j = 0; j < nonPinnedSlots; j++) {
            visible.push(cycleLogos[(cursor + j) % cycleLogos.length]);
        }
        cursor = (cursor + 1) % cycleLogos.length;
        var nextSet = pinnedLogos.concat(visible); // always includes the pinned logo(s)
        var current = slots.map(function(s) { return s.logo; });
        var arrangement = arrangeWithoutFixedPoints(nextSet, current);
        // Crossfade all slots at once — every slot's logo changed, so the whole wall animates.
        for (var i = 0; i < slots.length; i++) { setSlotLogo(slots[i], arrangement[i]); }
    }

    var INTERVAL_MS = 3500;
    var timer = setInterval(tick, INTERVAL_MS);

    // Pause when the tab is hidden — avoids wasted work and a janky catch-up burst.
    document.addEventListener('visibilitychange', function() {
        if (document.hidden) {
            if (timer) { clearInterval(timer); timer = null; }
        } else if (!timer) {
            timer = setInterval(tick, INTERVAL_MS);
        }
    });
});
