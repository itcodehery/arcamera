const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ---------------------------------------------------------------------------
// DOM References
// ---------------------------------------------------------------------------
const img = document.getElementById('camera-feed');
const captureBtn = document.getElementById('capture-btn');
const toast = document.getElementById('toast');
const modePhotoBtn = document.getElementById('mode-photo');
const modeVideoBtn = document.getElementById('mode-video');
const modeToggleContainer = document.getElementById('mode-toggle-container');

// Gallery
const galleryBtn = document.getElementById('gallery-btn');
const galleryPanel = document.getElementById('gallery-panel');
const galleryCloseBtn = document.getElementById('gallery-close-btn');
const galleryGrid = document.getElementById('gallery-grid');
const galleryEmpty = document.getElementById('gallery-empty');

// Settings
const settingsBtn = document.getElementById('settings-btn');
const settingsPanel = document.getElementById('settings-panel');
const settingsCloseBtn = document.getElementById('settings-close-btn');
const mirrorToggle = document.getElementById('mirror-toggle');
const gridToggle = document.getElementById('grid-toggle');

// Lightbox
const lightbox = document.getElementById('lightbox');
const lightboxImage = document.getElementById('lightbox-image');
const lightboxTitle = document.getElementById('lightbox-title');
const lightboxClose = document.getElementById('lightbox-close');
const lightboxDelete = document.getElementById('lightbox-delete');
const lightboxOpenExt = document.getElementById('lightbox-open-ext');

// Grid overlay
const gridOverlay = document.getElementById('grid-overlay');

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------
let currentFrameBase64 = null;
let currentMode = 'photo';
let isRecording = false;
let deleteConfirmTimer = null;

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------
async function startCamera() {
    await listen('camera-frame', (event) => {
        const base64Str = event.payload;
        currentFrameBase64 = "data:image/jpeg;base64," + base64Str;
        img.src = currentFrameBase64;
    });
}

// ---------------------------------------------------------------------------
// Toast
// ---------------------------------------------------------------------------
let toastTimer = null;
function showToast(message) {
    clearTimeout(toastTimer);
    toast.textContent = message;
    toast.classList.add('show');
    toastTimer = setTimeout(() => {
        toast.classList.remove('show');
    }, 3000);
}

// ---------------------------------------------------------------------------
// Mode Toggle
// ---------------------------------------------------------------------------
modePhotoBtn.addEventListener('click', () => {
    if (isRecording) return;
    currentMode = 'photo';
    modePhotoBtn.classList.add('active');
    modeVideoBtn.classList.remove('active');
    modeToggleContainer.classList.remove('mode-video-active');
});

modeVideoBtn.addEventListener('click', () => {
    if (isRecording) return;
    currentMode = 'video';
    modeVideoBtn.classList.add('active');
    modePhotoBtn.classList.remove('active');
    modeToggleContainer.classList.add('mode-video-active');
});

// ---------------------------------------------------------------------------
// Capture Button
// ---------------------------------------------------------------------------
captureBtn.addEventListener('click', async () => {
    if (currentMode === 'photo') {
        if (!currentFrameBase64) return;

        // Flash effect
        const flash = document.createElement('div');
        flash.style.position = 'absolute';
        flash.style.inset = '0';
        flash.style.backgroundColor = 'white';
        flash.style.opacity = '0.8';
        flash.style.transition = 'opacity 0.2s ease-out';
        flash.style.pointerEvents = 'none';
        flash.style.zIndex = '20';
        document.querySelector('.camera-container').appendChild(flash);

        setTimeout(() => {
            flash.style.opacity = '0';
            setTimeout(() => flash.remove(), 200);
        }, 50);

        try {
            await invoke('save_photo', { dataUrl: currentFrameBase64 });
            showToast('Photo saved!');
        } catch (error) {
            console.error('Save failed:', error);
            showToast('Failed to save photo');
        }
    } else if (currentMode === 'video') {
        if (!isRecording) {
            startRecording();
        } else {
            stopRecording();
        }
    }
});

// ---------------------------------------------------------------------------
// Video Recording (Rust backend)
// ---------------------------------------------------------------------------
async function startRecording() {
    try {
        await invoke('start_recording');
        isRecording = true;
        captureBtn.classList.add('recording');
    } catch (e) {
        showToast('Failed to start recording: ' + e);
    }
}

async function stopRecording() {
    isRecording = false;
    captureBtn.classList.remove('recording');
    showToast('Processing video...');

    try {
        await invoke('stop_recording');
        showToast('Video saved!');
    } catch (e) {
        showToast('Error: ' + e);
    }
}

// ---------------------------------------------------------------------------
// Gallery
// ---------------------------------------------------------------------------
galleryBtn.addEventListener('click', () => {
    galleryPanel.classList.add('open');
    loadGallery();
});

galleryCloseBtn.addEventListener('click', () => {
    galleryPanel.classList.remove('open');
});

async function loadGallery() {
    galleryGrid.innerHTML = '';
    galleryEmpty.style.display = 'none';
    galleryGrid.style.display = 'grid';

    try {
        const items = await invoke('list_gallery_items');

        if (items.length === 0) {
            galleryEmpty.style.display = 'flex';
            galleryGrid.style.display = 'none';
            return;
        }

        for (const item of items) {
            const el = createGalleryItem(item);
            galleryGrid.appendChild(el);
            loadThumbnail(el, item);
        }
    } catch (e) {
        console.error('Failed to load gallery:', e);
        showToast('Failed to load gallery');
    }
}

function createGalleryItem(item) {
    const div = document.createElement('div');
    div.className = 'gallery-item loading';

    const thumbImg = document.createElement('img');
    thumbImg.className = 'gallery-thumb';
    thumbImg.alt = item.filename;
    div.appendChild(thumbImg);

    if (item.is_video) {
        const badge = document.createElement('div');
        badge.className = 'video-badge';
        badge.innerHTML = '<span class="material-symbols-outlined">play_arrow</span>';
        div.appendChild(badge);
    }

    div.addEventListener('click', () => {
        if (item.is_video) {
            invoke('open_file_externally', { path: item.path }).catch(e => {
                showToast('Failed to open video: ' + e);
            });
        } else {
            openLightbox(item);
        }
    });

    return div;
}

async function loadThumbnail(el, item) {
    try {
        const base64 = await invoke('get_thumbnail', { path: item.path });
        const thumbImg = el.querySelector('.gallery-thumb');
        thumbImg.src = 'data:image/jpeg;base64,' + base64;
        thumbImg.onload = () => {
            thumbImg.classList.add('loaded');
            el.classList.remove('loading');
        };
    } catch (e) {
        el.classList.remove('loading');
        console.error('Thumbnail failed for', item.filename, e);
    }
}

// ---------------------------------------------------------------------------
// Lightbox
// ---------------------------------------------------------------------------
function openLightbox(item) {
    lightboxTitle.textContent = item.filename;
    lightboxImage.src = '';
    lightboxImage.classList.remove('loaded');
    lightbox.classList.add('open');
    lightbox.dataset.path = item.path;

    // Reset delete confirmation state
    resetDeleteConfirm();

    // Load full-resolution image
    invoke('get_thumbnail', { path: item.path }).then(base64 => {
        lightboxImage.src = 'data:image/jpeg;base64,' + base64;
        lightboxImage.onload = () => lightboxImage.classList.add('loaded');
    }).catch(e => {
        showToast('Failed to load image');
        console.error(e);
    });
}

lightboxClose.addEventListener('click', () => {
    lightbox.classList.remove('open');
    lightboxImage.src = '';
    lightboxImage.classList.remove('loaded');
});

lightboxOpenExt.addEventListener('click', () => {
    const path = lightbox.dataset.path;
    if (path) {
        invoke('open_file_externally', { path }).catch(e => {
            showToast('Failed to open file: ' + e);
        });
    }
});

// Delete with confirmation (tap twice)
lightboxDelete.addEventListener('click', async () => {
    const path = lightbox.dataset.path;
    if (!path) return;

    if (lightboxDelete.dataset.confirm !== 'true') {
        // First tap — enter confirm state
        lightboxDelete.dataset.confirm = 'true';
        lightboxDelete.classList.add('confirm-delete');
        lightboxDelete.querySelector('.material-symbols-outlined').textContent = 'delete_forever';

        clearTimeout(deleteConfirmTimer);
        deleteConfirmTimer = setTimeout(() => resetDeleteConfirm(), 3000);
        return;
    }

    // Second tap — actually delete
    resetDeleteConfirm();

    try {
        await invoke('delete_gallery_item', { path });
        lightbox.classList.remove('open');
        lightboxImage.src = '';
        lightboxImage.classList.remove('loaded');
        showToast('File deleted');
        loadGallery();
    } catch (e) {
        showToast('Failed to delete: ' + e);
    }
});

function resetDeleteConfirm() {
    clearTimeout(deleteConfirmTimer);
    lightboxDelete.dataset.confirm = '';
    lightboxDelete.classList.remove('confirm-delete');
    lightboxDelete.querySelector('.material-symbols-outlined').textContent = 'delete';
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------
settingsBtn.addEventListener('click', () => {
    settingsPanel.classList.add('open');
});

settingsCloseBtn.addEventListener('click', () => {
    settingsPanel.classList.remove('open');
});

// Mirror toggle
mirrorToggle.addEventListener('change', () => {
    img.style.transform = mirrorToggle.checked ? 'scaleX(-1)' : 'none';
    localStorage.setItem('arcamera-mirror', mirrorToggle.checked);
});

// Grid toggle
gridToggle.addEventListener('change', () => {
    gridOverlay.style.display = gridToggle.checked ? 'block' : 'none';
    localStorage.setItem('arcamera-grid', gridToggle.checked);
});

// Restore persisted settings
function loadSettings() {
    const mirror = localStorage.getItem('arcamera-mirror');
    if (mirror !== null) {
        mirrorToggle.checked = mirror === 'true';
        img.style.transform = mirrorToggle.checked ? 'scaleX(-1)' : 'none';
    }

    const grid = localStorage.getItem('arcamera-grid');
    if (grid !== null) {
        gridToggle.checked = grid === 'true';
        gridOverlay.style.display = gridToggle.checked ? 'block' : 'none';
    }
}

// ---------------------------------------------------------------------------
// Initialize
// ---------------------------------------------------------------------------
loadSettings();
startCamera();
