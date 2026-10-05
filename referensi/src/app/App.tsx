import { useState } from 'react';
import { AppProvider } from './lib/AppContext';
import LandingPage from './components/LandingPage';
import LoginPage from './components/LoginPage';
import AdminDashboard from './components/admin/AdminDashboard';
import SchoolDashboard from './components/school/SchoolDashboard';
import StudentDashboard from './components/student/StudentDashboard';
import ContentDashboard from './components/content/ContentDashboard';
import { Toaster } from './components/ui/sonner';

type UserRole = 'admin' | 'school' | 'student' | 'content';
type AppView = 'landing' | 'login' | 'dashboard';

interface User {
  id: string;
  name: string;
  email: string;
  role: UserRole;
  schoolName?: string;
  students?: number;
}

function AppContent() {
  const [view, setView] = useState<AppView>('landing');
  const [user, setUser] = useState<User | null>(null);

  const handleLogin = (userData: User) => {
    setUser(userData);
    setView('dashboard');
  };

  const handleLogout = () => {
    setUser(null);
    setView('landing');
  };

  if (view === 'landing') {
    return (
      <LandingPage
        onGetStarted={() => setView('login')}
        onLogin={() => setView('login')}
      />
    );
  }

  if (view === 'login' || !user) {
    return <LoginPage onLogin={handleLogin} onBack={() => setView('landing')} />;
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100">
      {user.role === 'admin' && <AdminDashboard onLogout={handleLogout} userData={user} />}
      {user.role === 'school' && <SchoolDashboard onLogout={handleLogout} userData={user} />}
      {user.role === 'student' && <StudentDashboard onLogout={handleLogout} userData={user} />}
      {user.role === 'content' && <ContentDashboard onLogout={handleLogout} userData={user} />}
    </div>
  );
}

export default function App() {
  return (
    <AppProvider>
      <AppContent />
      <Toaster />
    </AppProvider>
  );
}
