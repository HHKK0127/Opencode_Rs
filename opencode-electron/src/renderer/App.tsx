import { Show, createSignal, Match, Switch } from 'solid-js';
import { authState } from './store/auth';
import LoginForm from './components/LoginForm';
import DashboardPage from './pages/DashboardPage';
import EditorPage from './pages/EditorPage';

type Page = { name: string; params?: Record<string, string> };
const [currentPage, setCurrentPage] = createSignal<Page>({ name: 'dashboard' });

function AuthenticatedApp() {
  const handleNavigate = (page: string, params?: Record<string, string>) => {
    setCurrentPage({ name: page, params: params || {} });
  };

  return (
    <Switch>
      <Match when={currentPage().name === 'dashboard'}>
        <DashboardPage onNavigate={handleNavigate} />
      </Match>
      <Match when={currentPage().name === 'editor'}>
        <EditorPage
          params={currentPage().params}
          onNavigate={handleNavigate}
        />
      </Match>
    </Switch>
  );
}

function App() {
  return (
    <Show when={authState.isAuthenticated} fallback={<LoginForm />}>
      <AuthenticatedApp />
    </Show>
  );
}

export default App;