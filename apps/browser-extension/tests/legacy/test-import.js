// Test import of WebRTCManager
import('./src/entrypoints/offscreen/webrtcManager.ts')
    .then(module => {
        console.log('Import successful!');
        console.log('Exports:', Object.keys(module));
        console.log('WebRTCManager:', typeof module.WebRTCManager);
    })
    .catch(error => {
        console.error('Import failed:', error.message);
    });
