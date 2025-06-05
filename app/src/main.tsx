import React from 'react';
import ReactDOM from 'react-dom/client';

function App() {
  return <div className="p-2">OpenWarp</div>;
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
