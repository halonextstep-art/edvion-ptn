import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { 
  Users, 
  School, 
  FileText, 
  TrendingUp, 
  Activity,
  ArrowUp,
  ArrowDown,
  Clock,
  CheckCircle2
} from 'lucide-react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line, PieChart, Pie, Cell } from 'recharts';
import { useApp } from '../../lib/AppContext';
import { mockAnalytics } from '../../lib/mockData';

const performanceData = [
  { name: 'Sen', users: 4200 },
  { name: 'Sel', users: 5100 },
  { name: 'Rab', users: 4800 },
  { name: 'Kam', users: 6200 },
  { name: 'Jum', users: 7100 },
  { name: 'Sab', users: 8300 },
  { name: 'Min', users: 7500 },
];

const subjectData = [
  { name: 'TPS', value: 35, color: '#6366f1' },
  { name: 'Literasi', value: 28, color: '#8b5cf6' },
  { name: 'Matematika', value: 22, color: '#ec4899' },
  { name: 'Penalaran', value: 15, color: '#f59e0b' },
];

interface AdminOverviewProps {
  onNavigate?: (tab: string) => void;
}

export default function AdminOverview({ onNavigate }: AdminOverviewProps) {
  const { students, schools, questions, events } = useApp();
  const analytics = mockAnalytics;
  
  const recentEvents = events.slice(0, 4);
  const activeStudents = students.filter(s => {
    const lastActive = new Date(s.lastActive);
    const today = new Date();
    const diffDays = Math.floor((today.getTime() - lastActive.getTime()) / (1000 * 60 * 60 * 24));
    return diffDays <= 7;
  }).length;

  return (
    <div className="space-y-6">
      {/* Quick Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card className="p-6 border-l-4 border-l-indigo-500">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Total Siswa Aktif</p>
              <h3 className="text-3xl mb-2">{analytics.national.activeStudents.toLocaleString()}</h3>
              <div className="flex items-center gap-1 text-sm text-green-600">
                <ArrowUp className="w-4 h-4" />
                <span>12.5% vs bulan lalu</span>
              </div>
            </div>
            <div className="w-12 h-12 rounded-xl bg-indigo-100 flex items-center justify-center">
              <Users className="w-6 h-6 text-indigo-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6 border-l-4 border-l-purple-500">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Sekolah Mitra</p>
              <h3 className="text-3xl mb-2">{schools.length}</h3>
              <div className="flex items-center gap-1 text-sm text-green-600">
                <ArrowUp className="w-4 h-4" />
                <span>8.3% vs bulan lalu</span>
              </div>
            </div>
            <div className="w-12 h-12 rounded-xl bg-purple-100 flex items-center justify-center">
              <School className="w-6 h-6 text-purple-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6 border-l-4 border-l-pink-500">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Bank Soal</p>
              <h3 className="text-3xl mb-2">{analytics.national.totalQuestions.toLocaleString()}</h3>
              <div className="flex items-center gap-1 text-sm text-green-600">
                <ArrowUp className="w-4 h-4" />
                <span>{questions.length} tersedia</span>
              </div>
            </div>
            <div className="w-12 h-12 rounded-xl bg-pink-100 flex items-center justify-center">
              <FileText className="w-6 h-6 text-pink-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6 border-l-4 border-l-orange-500">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Event Aktif</p>
              <h3 className="text-3xl mb-2">{events.filter(e => e.status !== 'completed').length}</h3>
              <div className="flex items-center gap-1 text-sm text-orange-600">
                <Activity className="w-4 h-4" />
                <span>{events.filter(e => e.status === 'upcoming').length} scheduled</span>
              </div>
            </div>
            <div className="w-12 h-12 rounded-xl bg-orange-100 flex items-center justify-center">
              <TrendingUp className="w-6 h-6 text-orange-600" />
            </div>
          </div>
        </Card>
      </div>

      {/* Charts Row */}
      <div className="grid lg:grid-cols-3 gap-6">
        <Card className="lg:col-span-2 p-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg mb-1">Aktivitas Pengguna (7 Hari Terakhir)</h3>
              <p className="text-sm text-muted-foreground">Total user aktif per hari</p>
            </div>
            <Button variant="outline" size="sm">View Details</Button>
          </div>
          
          <ResponsiveContainer width="100%" height={280}>
            <BarChart data={performanceData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
              <XAxis dataKey="name" stroke="#888" />
              <YAxis stroke="#888" />
              <Tooltip 
                contentStyle={{ backgroundColor: '#fff', border: '1px solid #e5e7eb', borderRadius: '8px' }}
              />
              <Bar dataKey="users" fill="url(#colorGradient)" radius={[8, 8, 0, 0]} />
              <defs>
                <linearGradient id="colorGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#6366f1" />
                  <stop offset="100%" stopColor="#8b5cf6" />
                </linearGradient>
              </defs>
            </BarChart>
          </ResponsiveContainer>
        </Card>

        <Card className="p-6">
          <h3 className="text-lg mb-1">Distribusi Mata Uji</h3>
          <p className="text-sm text-muted-foreground mb-6">Popularitas per kategori</p>
          
          <ResponsiveContainer width="100%" height={280}>
            <PieChart>
              <Pie
                data={subjectData}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={100}
                paddingAngle={5}
                dataKey="value"
              >
                {subjectData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
          
          <div className="space-y-2 mt-4">
            {subjectData.map((item, idx) => (
              <div key={idx} className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full" style={{ backgroundColor: item.color }}></div>
                  <span>{item.name}</span>
                </div>
                <span>{item.value}%</span>
              </div>
            ))}
          </div>
        </Card>
      </div>

      {/* Recent Events */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-lg mb-1">Event Terkini</h3>
            <p className="text-sm text-muted-foreground">Status dan performa event</p>
          </div>
          <Button variant="outline" size="sm" onClick={() => onNavigate?.('events')}>Lihat Semua</Button>
        </div>

        <div className="space-y-3">
          {recentEvents.map((event) => (
            <div key={event.id} className="flex items-center justify-between p-4 bg-slate-50 rounded-lg hover:bg-slate-100 transition-colors">
              <div className="flex items-center gap-4">
                <div className={`w-2 h-2 rounded-full ${
                  event.status === 'ongoing' ? 'bg-green-500' :
                  event.status === 'completed' ? 'bg-blue-500' :
                  'bg-orange-500'
                }`}></div>
                <div>
                  <h4 className="mb-1">{event.title}</h4>
                  <div className="flex items-center gap-3 text-sm text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Users className="w-3.5 h-3.5" />
                      {event.participants.toLocaleString()} peserta
                    </span>
                    <span className="px-2 py-0.5 bg-white rounded text-xs">{event.type}</span>
                  </div>
                </div>
              </div>
              
              <div className="flex items-center gap-2">
                {event.status === 'ongoing' && (
                  <div className="flex items-center gap-1 text-sm text-green-600 bg-green-50 px-3 py-1.5 rounded-full">
                    <Activity className="w-3.5 h-3.5" />
                    <span>Berlangsung</span>
                  </div>
                )}
                {event.status === 'completed' && (
                  <div className="flex items-center gap-1 text-sm text-blue-600 bg-blue-50 px-3 py-1.5 rounded-full">
                    <CheckCircle2 className="w-3.5 h-3.5" />
                    <span>Selesai</span>
                  </div>
                )}
                {event.status === 'upcoming' && (
                  <div className="flex items-center gap-1 text-sm text-orange-600 bg-orange-50 px-3 py-1.5 rounded-full">
                    <Clock className="w-3.5 h-3.5" />
                    <span>Dijadwalkan</span>
                  </div>
                )}
                <Button variant="ghost" size="sm" onClick={() => onNavigate?.('events')}>Detail</Button>
              </div>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}