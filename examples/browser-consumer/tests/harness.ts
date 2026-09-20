import { loadRuntime, MixtureRuntimeError } from '@openmixture/runtime';
import { readMaterial, renderMaterial } from '../src/consumer';

const sdk = { loadRuntime, MixtureRuntimeError, readMaterial, renderMaterial };
declare global { interface Window { sdk: typeof sdk; } }
window.sdk = sdk;
