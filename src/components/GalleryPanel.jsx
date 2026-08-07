import './GalleryPanel.css';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Lightbox from './Lightbox';

export default function GalleryPanel({ isOpen, onClose, triggerToast }) {
    const [items, setItems] = useState([]);
    const [isLoading, setIsLoading] = useState(false);
    
    const [lightboxItem, setLightboxItem] = useState(null);

    const loadGallery = async () => {
        setIsLoading(true);
        try {
            const result = await invoke('list_gallery_items');
            setItems(result);
        } catch (e) {
            console.error('Failed to load gallery:', e);
            triggerToast('Failed to load gallery');
        }
        setIsLoading(false);
    };

    useEffect(() => {
        if (isOpen) {
            loadGallery();
        }
    }, [isOpen]);

    const handleItemClick = async (item) => {
        if (item.is_video) {
            try {
                await invoke('open_file_externally', { path: item.path });
            } catch (e) {
                triggerToast('Failed to open video: ' + e);
            }
        } else {
            setLightboxItem(item);
        }
    };

    const handleDelete = async (path) => {
        try {
            await invoke('delete_gallery_item', { path });
            triggerToast('File deleted');
            setLightboxItem(null);
            loadGallery();
        } catch (e) {
            triggerToast('Failed to delete: ' + e);
        }
    };

    return (
        <div className={`panel-overlay ${isOpen ? 'open' : ''}`}>
            <header className="panel-header">
                <button className="icon-button transparent-btn" onClick={onClose}>
                    <span className="material-symbols-outlined">arrow_back</span>
                </button>
                <h2 className="panel-title">Gallery</h2>
                <div style={{ width: '48px' }}></div>
            </header>
            
            {items.length === 0 && !isLoading ? (
                <div className="gallery-empty">
                    <span className="material-symbols-outlined" style={{ fontSize: '72px', opacity: 0.3 }}>photo_library</span>
                    <p>No photos or videos yet</p>
                    <p className="gallery-empty-sub">Your captures will appear here</p>
                </div>
            ) : (
                <div className="gallery-grid">
                    {items.map((item, index) => (
                        <GalleryItem 
                            key={item.path || index} 
                            item={item} 
                            onClick={() => handleItemClick(item)} 
                        />
                    ))}
                </div>
            )}

            {lightboxItem && (
                <Lightbox 
                    item={lightboxItem} 
                    onClose={() => setLightboxItem(null)} 
                    onDelete={() => handleDelete(lightboxItem.path)}
                    triggerToast={triggerToast}
                />
            )}
        </div>
    );
}

function GalleryItem({ item, onClick }) {
    const [base64, setBase64] = useState('');
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        invoke('get_thumbnail', { path: item.path }).then(res => {
            setBase64(res);
            setLoading(false);
        }).catch(e => {
            console.error('Thumbnail failed', e);
            setLoading(false);
        });
    }, [item.path]);

    return (
        <div className={`gallery-item ${loading ? 'loading' : ''}`} onClick={onClick}>
            {base64 && <img className="gallery-thumb loaded" src={`data:image/jpeg;base64,${base64}`} alt={item.filename} />}
            {item.is_video && (
                <div className="video-badge">
                    <span className="material-symbols-outlined">play_arrow</span>
                </div>
            )}
        </div>
    );
}
