const MATERIALS = [
  { id: 0, name: "Empty", phase: "Void", color: "#0b0e14", desc: "Vacuum / air cell (right-click erases)" },
  { id: 1, name: "Sand", phase: "Powder", color: "#d8ba72", desc: "Heavy granular sand; sinks cleanly in liquids" },
  { id: 2, name: "Gunpowder", phase: "Powder", color: "#44424a", desc: "Explosive powder; detonates at >=220°C or on Spark/Fire" },
  { id: 3, name: "Virus", phase: "Powder", color: "#a838d6", desc: "Bio-spore; infects Plant & Wood until burned by Fire/Acid" },
  { id: 4, name: "Oil", phase: "Liquid", color: "#6e4a22", desc: "Light hydrocarbon; floats on Water, ignites >=240°C" },
  { id: 5, name: "Water", phase: "Liquid", color: "#2a86e8", desc: "Boils >=100°C to Steam, freezes <=0°C, grows Plant" },
  { id: 6, name: "Saltwater", phase: "Liquid", color: "#3ea8d4", desc: "Dense brine; sinks under Water, conducts Spark strongly" },
  { id: 7, name: "Acid", phase: "Liquid", color: "#74f230", desc: "Dissolves Wood, Metal, Stone & Plant into ToxicGas" },
  { id: 8, name: "Nitro", phase: "Liquid", color: "#e4486c", desc: "Volatile nitroglycerin; shockwave blast on heat/Spark/Acid" },
  { id: 9, name: "Lava", phase: "Liquid", color: "#ff6612", desc: "1150°C magma; boils Water, melts Ice, quenches to Obsidian" },
  { id: 10, name: "Steam", phase: "Gas", color: "#d0e2f2", desc: "Rises, cools aloft <=82°C, condenses back into Water rain" },
  { id: 11, name: "Smoke", phase: "Gas", color: "#585c66", desc: "Combustion plume that disperses and fades" },
  { id: 12, name: "Methane", phase: "Gas", color: "#b086d4", desc: "Light flammable gas; flashes into fireball on Spark/Fire" },
  { id: 13, name: "ToxicGas", phase: "Gas", color: "#94d648", desc: "Corrosive vapor released during Acid dissolution" },
  { id: 14, name: "Ice", phase: "Solid", color: "#98dcf8", desc: "-20°C solid; chills Steam/Lava and melts >=2°C" },
  { id: 15, name: "Wood", phase: "Solid", color: "#80502c", desc: "Flammable timber; dissolved by Acid, infected by Virus" },
  { id: 16, name: "Plant", phase: "Solid", color: "#38ae4c", desc: "Living flora; grows when touching Water" },
  { id: 17, name: "Stone", phase: "Solid", color: "#727680", desc: "Refractory rock; dissolved slowly by Acid" },
  { id: 18, name: "Obsidian", phase: "Solid", color: "#2c223e", desc: "Acid-proof volcanic glass formed by quenched Lava" },
  { id: 19, name: "Metal", phase: "Solid", color: "#a4aebc", desc: "High-conductivity wire; carries Spark instantaneously via BFS" },
  { id: 20, name: "Fire", phase: "Energy", color: "#ff941c", desc: "650°C flame; ignites fuels, quenched by Water" },
  { id: 21, name: "Spark", phase: "Energy", color: "#82e8ff", desc: "Electrical arc; triggers Metal BFS circuits & explosives" },
];

const SPECIAL_TOOLS = [
  { id: "heat", name: "Heat +120°", color: "#ff5522", desc: "Thermal injector brush (+120°C)" },
  { id: "cryo", name: "Cryo -80°", color: "#4cc9ff", desc: "Cryogenic chiller brush (-80°C)" },
  { id: "erase", name: "Erase", color: "#1c2330", desc: "Clear cells to Empty" },
];

const PHASE_GROUPS = [
  { label: "POWDERS", ids: [1, 2, 3] },
  { label: "LIQUIDS", ids: [5, 6, 4, 7, 8, 9] },
  { label: "GASES", ids: [10, 11, 12, 13] },
  { label: "SOLIDS & ENERGY", ids: [17, 18, 19, 15, 16, 14, 20, 21] },
];

const state = {
  wasm: null,
  width: 320,
  height: 200,
  paused: false,
  speed: 1,
  speedAccum: 0,
  viewMode: 0,
  brushSize: 4,
  activeTool: { kind: "mat", id: 1 }, // Default Sand
  activePreset: 0,
  drawing: false,
  erasing: false,
  lastCell: null,
  hoverCell: null,
  fpsFrames: 0,
  fpsLast: performance.now(),
  fpsValue: 0,
};

const els = {
  simCanvas: document.getElementById("sim-canvas"),
  overlayCanvas: document.getElementById("overlay-canvas"),
  errorBanner: document.getElementById("wasm-error"),
  toolGrid: document.getElementById("tool-grid"),
  paletteGroups: document.getElementById("palette-groups"),
  brushSize: document.getElementById("brush-size"),
  brushSizeLabel: document.getElementById("brush-size-label"),
  btnPlay: document.getElementById("btn-play"),
  btnStep: document.getElementById("btn-step"),
  btnView: document.getElementById("btn-view"),
  btnReset: document.getElementById("btn-reset"),
  btnClear: document.getElementById("btn-clear"),
  inspCoord: document.getElementById("insp-coord"),
  inspSwatch: document.getElementById("insp-swatch"),
  inspMat: document.getElementById("insp-mat"),
  inspPhase: document.getElementById("insp-phase"),
  inspTemp: document.getElementById("insp-temp"),
  statFps: document.getElementById("stat-fps"),
  statTick: document.getElementById("stat-tick"),
  statActive: document.getElementById("stat-active"),
  statReactions: document.getElementById("stat-reactions"),
  statAvgTemp: document.getElementById("stat-avg-temp"),
  statMaxTemp: document.getElementById("stat-max-temp"),
  segPowder: document.getElementById("seg-powder"),
  segLiquid: document.getElementById("seg-liquid"),
  segGas: document.getElementById("seg-gas"),
  segSolid: document.getElementById("seg-solid"),
  phaseLegend: document.getElementById("phase-legend"),
};

const simCtx = els.simCanvas.getContext("2d", { alpha: false });
const overlayCtx = els.overlayCanvas.getContext("2d");

function showError(msg) {
  els.errorBanner.textContent = msg;
  els.errorBanner.hidden = false;
}

function buildPalette() {
  for (const tool of SPECIAL_TOOLS) {
    const btn = document.createElement("button");
    btn.className = "el-btn";
    btn.title = tool.desc;
    btn.dataset.toolKind = tool.id;
    btn.innerHTML = `<span class="swatch" style="background:${tool.color}"></span><span class="el-name">${tool.name}</span>`;
    btn.addEventListener("click", () => selectTool({ kind: tool.id, id: 0 }));
    els.toolGrid.appendChild(btn);
  }

  for (const group of PHASE_GROUPS) {
    const section = document.createElement("div");
    section.className = "phase-group";
    const header = document.createElement("div");
    header.className = "phase-header";
    header.textContent = group.label;
    section.appendChild(header);

    const grid = document.createElement("div");
    grid.className = "el-grid";
    for (const id of group.ids) {
      const m = MATERIALS[id];
      const btn = document.createElement("button");
      btn.className = "el-btn" + (id === state.activeTool.id ? " active" : "");
      btn.title = m.desc;
      btn.dataset.matId = String(id);
      btn.innerHTML = `<span class="swatch" style="background:${m.color}"></span><span class="el-name">${m.name}</span>`;
      btn.addEventListener("click", () => selectTool({ kind: "mat", id }));
      grid.appendChild(btn);
    }
    section.appendChild(grid);
    els.paletteGroups.appendChild(section);
  }
}

function selectTool(tool) {
  state.activeTool = tool;
  document.querySelectorAll(".el-btn").forEach((btn) => {
    const isMat = btn.dataset.matId !== undefined && tool.kind === "mat" && Number(btn.dataset.matId) === tool.id;
    const isSpecial = btn.dataset.toolKind !== undefined && btn.dataset.toolKind === tool.kind;
    btn.classList.toggle("active", isMat || isSpecial);
  });
}

async function loadWasm() {
  try {
    let instance;
    if (WebAssembly.instantiateStreaming) {
      try {
        const res = await WebAssembly.instantiateStreaming(fetch("./pixel_alchemy.wasm"), {});
        instance = res.instance;
      } catch {
        const bytes = await (await fetch("./pixel_alchemy.wasm")).arrayBuffer();
        const res = await WebAssembly.instantiate(bytes, {});
        instance = res.instance;
      }
    } else {
      const bytes = await (await fetch("./pixel_alchemy.wasm")).arrayBuffer();
      const res = await WebAssembly.instantiate(bytes, {});
      instance = res.instance;
    }

    state.wasm = instance.exports;
    state.wasm.wasm_init(20261118);
    state.width = state.wasm.wasm_width();
    state.height = state.wasm.wasm_height();
    els.simCanvas.width = state.width;
    els.simCanvas.height = state.height;
    els.overlayCanvas.width = state.width;
    els.overlayCanvas.height = state.height;
    state.wasm.wasm_load_preset(state.activePreset);
    renderFrame();
    requestAnimationFrame(loop);
  } catch (err) {
    showError(`Failed to load ./pixel_alchemy.wasm — serve via HTTP/HTTPS (${err.message})`);
  }
}

function canvasToCell(clientX, clientY) {
  const rect = els.simCanvas.getBoundingClientRect();
  const x = Math.floor(((clientX - rect.left) / rect.width) * state.width);
  const y = Math.floor(((clientY - rect.top) / rect.height) * state.height);
  return {
    x: Math.max(0, Math.min(state.width - 1, x)),
    y: Math.max(0, Math.min(state.height - 1, y)),
  };
}

function applyBrush(from, to, eraseMode) {
  if (!state.wasm) return;
  const r = state.brushSize;
  if (eraseMode || state.activeTool.kind === "erase") {
    state.wasm.wasm_paint_line(from.x, from.y, to.x, to.y, r, 0);
    return;
  }
  if (state.activeTool.kind === "heat") {
    state.wasm.wasm_thermal_brush(to.x, to.y, r, 120.0);
    return;
  }
  if (state.activeTool.kind === "cryo") {
    state.wasm.wasm_thermal_brush(to.x, to.y, r, -80.0);
    return;
  }
  state.wasm.wasm_paint_line(from.x, from.y, to.x, to.y, r, state.activeTool.id);
}

function updateInspector() {
  if (!state.wasm || !state.hoverCell) {
    els.inspCoord.textContent = "x: --- y: ---";
    els.inspMat.textContent = "Empty";
    els.inspPhase.textContent = "Void";
    els.inspSwatch.style.background = "#0b0e14";
    els.inspTemp.textContent = "22.0 °C";
    return;
  }
  const { x, y } = state.hoverCell;
  const matId = state.wasm.wasm_cell_material(x, y);
  const temp = state.wasm.wasm_cell_temp(x, y);
  const m = MATERIALS[matId] || MATERIALS[0];

  els.inspCoord.textContent = `x:${String(x).padStart(3, " ")} y:${String(y).padStart(3, " ")}`;
  els.inspMat.textContent = m.name;
  els.inspPhase.textContent = m.phase;
  els.inspSwatch.style.background = m.color;
  els.inspTemp.textContent = `${temp.toFixed(1)} °C`;
}

function drawOverlay() {
  overlayCtx.clearRect(0, 0, state.width, state.height);
  if (!state.hoverCell) return;
  const { x, y } = state.hoverCell;
  overlayCtx.strokeStyle =
    state.activeTool.kind === "heat"
      ? "rgba(255, 110, 50, 0.9)"
      : state.activeTool.kind === "cryo"
      ? "rgba(90, 210, 255, 0.9)"
      : "rgba(255, 255, 255, 0.72)";
  overlayCtx.lineWidth = 1;
  overlayCtx.beginPath();
  overlayCtx.arc(x + 0.5, y + 0.5, Math.max(0.8, state.brushSize), 0, Math.PI * 2);
  overlayCtx.stroke();
}

function renderFrame() {
  if (!state.wasm) return;
  const ptr = state.wasm.wasm_rgba_ptr();
  const len = state.wasm.wasm_rgba_len();
  const view = new Uint8ClampedArray(state.wasm.memory.buffer, ptr, len);
  const imgData = new ImageData(view, state.width, state.height);
  simCtx.putImageData(imgData, 0, 0);

  // Telemetry
  const active = state.wasm.wasm_active_count();
  els.statTick.textContent = state.wasm.wasm_tick_count().toLocaleString();
  els.statActive.textContent = active.toLocaleString();
  els.statReactions.textContent = state.wasm.wasm_reaction_count().toLocaleString();
  els.statAvgTemp.textContent = `${state.wasm.wasm_avg_temp().toFixed(1)} °C`;
  els.statMaxTemp.textContent = `${state.wasm.wasm_max_temp().toFixed(1)} °C`;

  const p = state.wasm.wasm_phase_count(1);
  const l = state.wasm.wasm_phase_count(2);
  const g = state.wasm.wasm_phase_count(3);
  const s = state.wasm.wasm_phase_count(4);
  const total = Math.max(1, p + l + g + s);
  els.segPowder.style.width = `${(p / total) * 100}%`;
  els.segLiquid.style.width = `${(l / total) * 100}%`;
  els.segGas.style.width = `${(g / total) * 100}%`;
  els.segSolid.style.width = `${(s / total) * 100}%`;
  els.phaseLegend.textContent = `P:${p} L:${l} G:${g} S:${s}`;

  updateInspector();
  drawOverlay();
}

function loop(now) {
  if (state.wasm) {
    if (!state.paused) {
      state.speedAccum += state.speed;
      const steps = Math.floor(state.speedAccum);
      if (steps >= 1) {
        state.speedAccum -= steps;
        state.wasm.wasm_step(Math.min(steps, 8));
      }
    }
    if (state.drawing && state.hoverCell && (state.activeTool.kind === "heat" || state.activeTool.kind === "cryo")) {
      applyBrush(state.hoverCell, state.hoverCell, false);
    }
    renderFrame();
  }

  state.fpsFrames += 1;
  if (now - state.fpsLast >= 400) {
    state.fpsValue = Math.round((state.fpsFrames * 1000) / (now - state.fpsLast));
    els.statFps.textContent = String(state.fpsValue);
    state.fpsFrames = 0;
    state.fpsLast = now;
  }
  requestAnimationFrame(loop);
}

function bindEvents() {
  els.simCanvas.addEventListener("contextmenu", (e) => e.preventDefault());

  els.simCanvas.addEventListener("pointerdown", (e) => {
    els.simCanvas.setPointerCapture(e.pointerId);
    const cell = canvasToCell(e.clientX, e.clientY);
    state.drawing = true;
    state.erasing = e.button === 2 || e.shiftKey;
    state.hoverCell = cell;
    state.lastCell = cell;
    applyBrush(cell, cell, state.erasing);
    renderFrame();
  });

  els.simCanvas.addEventListener("pointermove", (e) => {
    const cell = canvasToCell(e.clientX, e.clientY);
    state.hoverCell = cell;
    if (state.drawing) {
      const from = state.lastCell || cell;
      applyBrush(from, cell, state.erasing);
      state.lastCell = cell;
      renderFrame();
    } else {
      updateInspector();
      drawOverlay();
    }
  });

  window.addEventListener("pointerup", () => {
    state.drawing = false;
    state.erasing = false;
    state.lastCell = null;
  });

  els.simCanvas.addEventListener("pointerleave", () => {
    if (!state.drawing) {
      state.hoverCell = null;
      updateInspector();
      drawOverlay();
    }
  });

  els.brushSize.addEventListener("input", () => {
    state.brushSize = Number(els.brushSize.value);
    els.brushSizeLabel.textContent = `${state.brushSize} px`;
    drawOverlay();
  });

  document.querySelectorAll(".preset-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = Number(btn.dataset.preset);
      state.activePreset = id;
      document.querySelectorAll(".preset-btn").forEach((b) => b.classList.toggle("active", b === btn));
      if (state.wasm) {
        state.wasm.wasm_load_preset(id);
        renderFrame();
      }
    });
  });

  document.querySelectorAll(".speed-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      state.speed = Number(btn.dataset.speed);
      document.querySelectorAll(".speed-btn").forEach((b) => b.classList.toggle("active", b === btn));
    });
  });

  els.btnPlay.addEventListener("click", () => {
    state.paused = !state.paused;
    els.btnPlay.textContent = state.paused ? "Play" : "Pause";
    els.btnPlay.classList.toggle("active", state.paused);
  });

  els.btnStep.addEventListener("click", () => {
    if (!state.wasm) return;
    state.paused = true;
    els.btnPlay.textContent = "Play";
    els.btnPlay.classList.add("active");
    state.wasm.wasm_step(1);
    renderFrame();
  });

  els.btnView.addEventListener("click", () => {
    if (!state.wasm) return;
    state.viewMode = state.viewMode === 0 ? 1 : 0;
    state.wasm.wasm_set_view_mode(state.viewMode);
    els.btnView.textContent = state.viewMode === 0 ? "View: Material" : "View: Thermal IR";
    els.btnView.classList.toggle("active", state.viewMode === 1);
    renderFrame();
  });

  els.btnReset.addEventListener("click", () => {
    if (!state.wasm) return;
    state.wasm.wasm_load_preset(state.activePreset);
    renderFrame();
  });

  els.btnClear.addEventListener("click", () => {
    if (!state.wasm) return;
    state.wasm.wasm_clear();
    renderFrame();
  });

  window.addEventListener("keydown", (e) => {
    if (e.target.tagName === "INPUT") return;
    if (e.code === "Space") {
      e.preventDefault();
      els.btnPlay.click();
    } else if (e.key === ".") {
      els.btnStep.click();
    } else if (e.key === "t" || e.key === "T") {
      els.btnView.click();
    } else if (e.key === "r" || e.key === "R") {
      els.btnReset.click();
    } else if (e.key === "c" || e.key === "C") {
      els.btnClear.click();
    } else if (e.key === "[") {
      state.brushSize = Math.max(1, state.brushSize - 1);
      els.brushSize.value = String(state.brushSize);
      els.brushSizeLabel.textContent = `${state.brushSize} px`;
      drawOverlay();
    } else if (e.key === "]") {
      state.brushSize = Math.min(16, state.brushSize + 1);
      els.brushSize.value = String(state.brushSize);
      els.brushSizeLabel.textContent = `${state.brushSize} px`;
      drawOverlay();
    }
  });
}

buildPalette();
bindEvents();
loadWasm();
