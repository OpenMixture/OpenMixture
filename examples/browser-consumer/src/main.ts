import { loadRuntime, MixtureRuntimeError, type ChannelId } from '@openmixture/runtime';
import { formatReport, readMaterial, renderMaterial } from './consumer';
import './style.css';

const form = document.querySelector<HTMLFormElement>('#controls')!;
const status = document.querySelector<HTMLElement>('#status')!;
const outputs = document.querySelector<HTMLElement>('#outputs')!;
const report = document.querySelector<HTMLElement>('#report')!;
const controls = form.querySelectorAll<HTMLInputElement | HTMLButtonElement>('input, button');

form.addEventListener('submit', async event => {
  event.preventDefault();
  const data = new FormData(form);
  const channels = data.getAll('channel') as ChannelId[];
  if (channels.length === 0) { status.textContent = 'Select at least one channel.'; return; }
  controls.forEach(control => { control.disabled = true; });
  status.textContent = 'Loading and rendering…';
  outputs.replaceChildren();
  try {
    const source = await readMaterial(new URL(`${import.meta.env.BASE_URL}input.mix`, location.href));
    const runtime = await loadRuntime();
    const { build, inspection, result } = await renderMaterial(runtime, source, {
      size: [256, 128], channels,
      overrides: { frequency: Number(data.get('frequency')), roughness: Number(data.get('roughness')) },
    });
    // renderMaterial has already destroyed its GPU instance. These bytes remain owned.
    for (const channel of result.channels) {
      const figure = document.createElement('figure');
      const canvas = document.createElement('canvas');
      [canvas.width, canvas.height] = channel.size;
      canvas.setAttribute('aria-label', channel.channel);
      const context = canvas.getContext('2d');
      if (!context) throw new Error('A 2D canvas is required to display returned pixels.');
      context.putImageData(new ImageData(new Uint8ClampedArray(channel.pixels), ...channel.size), 0, 0);
      const caption = document.createElement('figcaption');
      caption.textContent = `${channel.channel} · ${channel.encoding}`;
      figure.append(canvas, caption);
      outputs.append(figure);
    }
    report.textContent = formatReport({ build, planHash: inspection.plan.hash,
      exposedParameters: inspection.exposedParameters, render: result.report });
    status.textContent = 'Rendered. GPU runtime destroyed; displayed pixels remain owned by this page.';
  } catch (error) {
    const detail = error instanceof MixtureRuntimeError
      ? { code: error.code, operation: error.operation, message: error.message,
        diagnostics: error.diagnostics, browserFailure: error.browserFailure, evidence: error.evidence }
      : { message: String(error) };
    status.textContent = 'Rendering failed. See the diagnostic report.';
    report.textContent = formatReport(detail);
    document.querySelector('details')!.open = true;
  } finally {
    controls.forEach(control => { control.disabled = false; });
  }
});
