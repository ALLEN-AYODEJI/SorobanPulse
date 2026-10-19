import { Routes, Route, Link } from 'react-router-dom'
import EventExplorer from './pages/EventExplorer'
import LiveStream from './pages/LiveStream'

function App() {
  return (
    <div style={{ fontFamily: 'system-ui, sans-serif' }}>
      <nav style={{ padding: '1rem', borderBottom: '1px solid #ccc' }}>
        <Link to="/" style={{ marginRight: '1rem' }}>
          Event Explorer
        </Link>
        <Link to="/live">Live Stream</Link>
      </nav>
      <Routes>
        <Route path="/" element={<EventExplorer />} />
        <Route path="/live" element={<LiveStream />} />
      </Routes>
    </div>
  )
}

export default App
