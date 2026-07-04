import './Sidebar.css'

export const Sidebar = () => {
  return (
    <aside data-component="sidebar">
      <div data-slot="sidebar-header">
        <h1>OpenCode</h1>
      </div>
      <div data-slot="session-list">
        <div data-slot="session-item" data-active="true">
          <span>New Chat</span>
        </div>
      </div>
      <div data-slot="sidebar-footer">
        <span>v0.1.0</span>
      </div>
    </aside>
  )
}
