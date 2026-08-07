import './CameraView.css';
import { useEffect, useRef, useState } from 'react';

export default function CameraView({ mirror, grid }) {
    const imgRef = useRef(null);
    const [flash, setFlash] = useState(false);

    useEffect(() => {
        // Simple trick to bypass cache if image stream gets stuck
        if (imgRef.current) {
            imgRef.current.onerror = () => {
                setTimeout(() => {
                    if (imgRef.current) {
                        imgRef.current.src = "http://127.0.0.1:49283/stream?" + new Date().getTime();
                    }
                }, 1000);
            };
        }
        
        const handleFlash = () => {
            setFlash(true);
            setTimeout(() => setFlash(false), 200);
        };
        
        window.addEventListener('arcamera-flash', handleFlash);
        return () => window.removeEventListener('arcamera-flash', handleFlash);
    }, []);

    return (
        <main className="camera-container">
            <img 
                ref={imgRef}
                id="camera-feed" 
                src="http://127.0.0.1:49283/stream" 
                style={{ transform: mirror ? 'scaleX(-1)' : 'none' }}
                alt="Camera Feed"
            />
            {grid && (
                <div className="grid-overlay" style={{ display: 'block' }}>
                    <div className="grid-line grid-h" style={{ top: '33.33%' }}></div>
                    <div className="grid-line grid-h" style={{ top: '66.66%' }}></div>
                    <div className="grid-line grid-v" style={{ left: '33.33%' }}></div>
                    <div className="grid-line grid-v" style={{ left: '66.66%' }}></div>
                </div>
            )}
            
            {flash && (
                <div className="flash-overlay"></div>
            )}
        </main>
    );
}
