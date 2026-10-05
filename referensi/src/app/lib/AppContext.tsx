import { createContext, useContext, useState, ReactNode, useCallback } from 'react';
import { mockStudents, mockSchools, mockEvents, mockQuestions, Student, School, Event, Question } from './mockData';

export type NotifRole = 'admin' | 'school' | 'student' | 'content' | 'all';
export type NotifType = 'info' | 'success' | 'warning' | 'review';

export interface Notification {
  id: string;
  title: string;
  message: string;
  type: NotifType;
  forRole: NotifRole;
  read: boolean;
  createdAt: string;
  link?: string;
}

interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'school' | 'student' | 'content';
  schoolName?: string;
  students?: number;
}

interface AppContextType {
  user: User | null;
  setUser: (user: User | null) => void;
  students: Student[];
  schools: School[];
  events: Event[];
  questions: Question[];
  updateStudent: (id: string, data: Partial<Student>) => void;
  updateSchool: (id: string, data: Partial<School>) => void;
  updateEvent: (id: string, data: Partial<Event>) => void;
  addQuestion: (question: Question) => void;
  updateQuestion: (id: string, data: Partial<Question>) => void;
  deleteQuestion: (id: string) => void;
  addEvent: (event: Event) => void;
  // Notifications
  notifications: Notification[];
  addNotification: (n: Omit<Notification, 'id' | 'createdAt' | 'read'>) => void;
  markRead: (id: string) => void;
  markAllRead: (role: NotifRole) => void;
  unreadCount: (role: NotifRole) => number;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

const SEED_NOTIFS: Notification[] = [
  { id: 'n1', title: 'Soal Baru Masuk Review', message: '3 soal dari Tim Konten menunggu persetujuan.', type: 'review', forRole: 'content', read: false, createdAt: '2025-01-23 09:00' },
  { id: 'n2', title: 'Mitra Baru Mendaftar', message: 'SMA Negeri 5 Bandung mengajukan pendaftaran mitra.', type: 'info', forRole: 'admin', read: false, createdAt: '2025-01-23 08:45' },
  { id: 'n3', title: 'Tryout Nasional #46 Dimulai', message: 'Tryout nasional sudah tersedia. Selamat mengerjakan!', type: 'success', forRole: 'student', read: false, createdAt: '2025-01-23 08:00' },
  { id: 'n4', title: 'Laporan Bulanan Siap', message: 'Laporan performa siswa bulan Januari sudah bisa diunduh.', type: 'success', forRole: 'school', read: false, createdAt: '2025-01-23 07:30' },
  { id: 'n5', title: 'Paket Drilling Baru', message: 'Paket "Drilling PM Level Sulit" sudah dipublish.', type: 'info', forRole: 'student', read: false, createdAt: '2025-01-22 15:00' },
  { id: 'n6', title: 'Pembayaran Diterima', message: 'Pembayaran dari SMA BPK Penabur berhasil diproses.', type: 'success', forRole: 'admin', read: true, createdAt: '2025-01-22 14:00' },
  { id: 'n7', title: 'Siswa Baru Ditambahkan', message: '12 siswa baru berhasil diimport dari file CSV.', type: 'success', forRole: 'school', read: true, createdAt: '2025-01-22 10:00' },
  { id: 'n8', title: 'Soal Ditolak', message: 'Soal SNBT-PU-008 ditolak reviewer. Silakan revisi.', type: 'warning', forRole: 'content', read: true, createdAt: '2025-01-21 16:00' },
];

export function AppProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [students, setStudents] = useState<Student[]>(mockStudents);
  const [schools, setSchools] = useState<School[]>(mockSchools);
  const [events, setEvents] = useState<Event[]>(mockEvents);
  const [questions, setQuestions] = useState<Question[]>(mockQuestions);
  const [notifications, setNotifications] = useState<Notification[]>(SEED_NOTIFS);

  const updateStudent = (id: string, data: Partial<Student>) =>
    setStudents(prev => prev.map(s => s.id === id ? { ...s, ...data } : s));
  const updateSchool = (id: string, data: Partial<School>) =>
    setSchools(prev => prev.map(s => s.id === id ? { ...s, ...data } : s));
  const updateEvent = (id: string, data: Partial<Event>) =>
    setEvents(prev => prev.map(e => e.id === id ? { ...e, ...data } : e));
  const addQuestion = (question: Question) => setQuestions(prev => [...prev, question]);
  const updateQuestion = (id: string, data: Partial<Question>) =>
    setQuestions(prev => prev.map(q => q.id === id ? { ...q, ...data } : q));
  const deleteQuestion = (id: string) => setQuestions(prev => prev.filter(q => q.id !== id));
  const addEvent = (event: Event) => setEvents(prev => [...prev, event]);

  const addNotification = useCallback((n: Omit<Notification, 'id' | 'createdAt' | 'read'>) => {
    const now = new Date();
    const createdAt = `${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}-${String(now.getDate()).padStart(2,'0')} ${String(now.getHours()).padStart(2,'0')}:${String(now.getMinutes()).padStart(2,'0')}`;
    setNotifications(prev => [{ ...n, id: `n${Date.now()}`, createdAt, read: false }, ...prev]);
  }, []);

  const markRead = useCallback((id: string) =>
    setNotifications(prev => prev.map(n => n.id === id ? { ...n, read: true } : n)), []);

  const markAllRead = useCallback((role: NotifRole) =>
    setNotifications(prev => prev.map(n => (n.forRole === role || n.forRole === 'all') ? { ...n, read: true } : n)), []);

  const unreadCount = useCallback((role: NotifRole) =>
    notifications.filter(n => !n.read && (n.forRole === role || n.forRole === 'all')).length, [notifications]);

  return (
    <AppContext.Provider value={{
      user, setUser, students, schools, events, questions,
      updateStudent, updateSchool, updateEvent,
      addQuestion, updateQuestion, deleteQuestion, addEvent,
      notifications, addNotification, markRead, markAllRead, unreadCount,
    }}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const context = useContext(AppContext);
  if (!context) throw new Error('useApp must be used within an AppProvider');
  return context;
}
