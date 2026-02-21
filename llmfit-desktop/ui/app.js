const invoke = window.__TAURI_INTERNALS__
  ? window.__TAURI_INTERNALS__.invoke
  : async (cmd) => { console.warn('Tauri not available, cmd:', cmd); return null; };

let allFits = [];

function esc(s) {
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

async function loadSpecs() {
  try {
    const specs = await invoke('get_system_specs');
    if (!specs) return;

    document.getElementById('cpu-name').textContent = specs.cpu_name;
    document.getElementById('cpu-cores').textContent = specs.cpu_cores + ' cores';
    document.getElementById('ram-total').textContent = specs.total_ram_gb.toFixed(1) + ' GB';
    document.getElementById('ram-available').textContent = specs.available_ram_gb.toFixed(1) + ' GB';

    // Render GPU cards
    const container = document.getElementById('gpus-container');
    container.innerHTML = '';

    if (specs.gpus.length === 0) {
      const card = document.createElement('div');
      card.className = 'spec-card';
      card.innerHTML = '<span class="spec-label">GPU</span>' +
        '<span class="spec-value">No GPU detected</span>';
      container.appendChild(card);
    } else {
      specs.gpus.forEach((gpu, i) => {
        const card = document.createElement('div');
        card.className = 'spec-card';
        const label = specs.gpus.length > 1 ? 'GPU ' + (i + 1) : 'GPU';
        const countStr = gpu.count > 1 ? ' ×' + gpu.count : '';
        const vramStr = gpu.vram_gb != null ? gpu.vram_gb.toFixed(1) + ' GB VRAM' : 'Shared memory';
        const backendStr = gpu.backend !== 'None' ? gpu.backend : '';
        const details = [vramStr, backendStr].filter(Boolean).join(' · ');
        card.innerHTML = '<span class="spec-label">' + esc(label) + '</span>' +
          '<span class="spec-value">' + esc(gpu.name + countStr) + '</span>' +
          '<span class="spec-detail">' + esc(details) + '</span>';
        container.appendChild(card);
      });
    }

    // Unified memory indicator
    if (specs.unified_memory) {
      const archCard = document.getElementById('memory-arch-card');
      archCard.style.display = '';
      document.getElementById('memory-arch').textContent = 'Unified (CPU + GPU shared)';
    }
  } catch (e) {
    console.error('Failed to load specs:', e);
    document.getElementById('cpu-name').textContent = 'Error loading specs';
  }
}

function fitClass(level) {
  switch (level) {
    case 'Perfect': return 'fit-perfect';
    case 'Good': return 'fit-good';
    case 'Marginal': return 'fit-marginal';
    default: return 'fit-tight';
  }
}

function modeClass(mode) {
  switch (mode) {
    case 'GPU': return 'mode-gpu';
    case 'MoE Offload': return 'mode-moe';
    case 'CPU Offload': return 'mode-cpuoffload';
    default: return 'mode-cpuonly';
  }
}

function renderModels(fits) {
  const tbody = document.getElementById('models-body');
  if (!fits || fits.length === 0) {
    tbody.innerHTML = '<tr><td colspan="9" class="loading">No models found</td></tr>';
    return;
  }
  tbody.innerHTML = fits.map(f => `
    <tr>
      <td><strong>${esc(f.name)}</strong></td>
      <td>${esc(f.params_b.toFixed(1))}B</td>
      <td>${esc(f.quant)}</td>
      <td class="${fitClass(f.fit_level)}">${esc(f.fit_level)}</td>
      <td class="${modeClass(f.run_mode)}">${esc(f.run_mode)}</td>
      <td>${esc(f.score.toFixed(0))}</td>
      <td>${esc(f.memory_required_gb.toFixed(1))} GB</td>
      <td>${esc(f.estimated_tps.toFixed(1))}</td>
      <td>${esc(f.use_case)}</td>
    </tr>
  `).join('');
}

function applyFilters() {
  const search = document.getElementById('search').value.toLowerCase();
  const fitFilter = document.getElementById('fit-filter').value;

  let filtered = allFits;
  if (search) {
    filtered = filtered.filter(f => f.name.toLowerCase().includes(search));
  }
  if (fitFilter !== 'all') {
    filtered = filtered.filter(f => f.fit_level === fitFilter);
  }
  renderModels(filtered);
}

async function loadModels() {
  try {
    allFits = await invoke('get_model_fits') || [];
    applyFilters();
  } catch (e) {
    console.error('Failed to load models:', e);
    document.getElementById('models-body').innerHTML =
      '<tr><td colspan="9" class="loading">Error loading models</td></tr>';
  }
}

document.getElementById('search').addEventListener('input', applyFilters);
document.getElementById('fit-filter').addEventListener('change', applyFilters);

loadSpecs();
loadModels();
