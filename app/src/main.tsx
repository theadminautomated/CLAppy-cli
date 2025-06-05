import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { motion } from 'framer-motion';
import logoLight from '../mustache-light.png';
import logoDark from '../mustache-dark.png';
import './index.css';

function App() {
  const [theme, setTheme] = useState<'light' | 'dark'>('dark');
  const [palette, setPalette] = useState(false);
  useEffect(() => {
    document.documentElement.className = theme;
  }, [theme]);
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setPalette((o) => !o);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);
  return (
    <div className="min-h-screen flex flex-col items-center">
      <motion.img
        src={theme === 'light' ? logoDark : logoLight}
        alt="CLAppy logo"
        className="w-16 h-16 mt-4 cursor-pointer"
        whileTap={{ rotate: 360 }}
        onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}
      />
      <div className="p-2">CLAppy</div>
      {palette && (
        <div className="fixed inset-0 bg-black/50 flex items-start justify-center pt-20" onClick={() => setPalette(false)}>
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-4 w-80" onClick={(e) => e.stopPropagation()}>
            <input autoFocus placeholder="Type a command" className="w-full p-2 rounded bg-gray-100 dark:bg-gray-900" />
          </div>
        </div>
      )}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
