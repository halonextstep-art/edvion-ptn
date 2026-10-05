import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { 
  Users, 
  TrendingUp, 
  Trophy, 
  Target,
  ArrowUp,
  Clock,
  Award,
  CheckCircle2
} from 'lucide-react';
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';

const weeklyData = [
  { day: 'Sen', active: 180 },
  { day: 'Sel', active: 195 },
  { day: 'Rab', active: 172 },
  { day: 'Kam', active: 210 },
  { day: 'Jum', active: 198 },
  { day: 'Sab', active: 145 },
  { day: 'Min', active: 132 },
];

const subjectPerformance = [
  { subject: 'TPS', avgScore: 72, color: '#6366f1' },
  { subject: 'Literasi', avgScore: 68, color: '#8b5cf6' },
  { subject: 'Matematika', avgScore: 64, color: '#ec4899' },
  { subject: 'Inggris', avgScore: 75, color: '#f59e0b' },
];

const topStudents = [
  { rank: 1, name: 'Ahmad Fauzi', score: 685, improvement: '+12' },
  { rank: 2, name: 'Siti Nurhaliza', score: 678, improvement: '+8' },
  { rank: 3, name: 'Budi Santoso', score: 672, improvement: '+15' },
  { rank: 4, name: 'Dewi Anggraini', score: 665, improvement: '+5' },
  { rank: 5, name: 'Rizki Pratama', score: 658, improvement: '+10' },
];

interface SchoolOverviewProps {
  schoolData?: any;
  onNavigate?: (tab: string) => void;
}

export default function SchoolOverview({ schoolData, onNavigate }: SchoolOverviewProps) {
  return (
    <div className="space-y-6">
      {/* Welcome Card */}
      <Card className="p-6 bg-gradient-to-br from-blue-600 to-cyan-600 text-white">
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div>
            <h2 className="text-2xl mb-2">Selamat Datang, {schoolData?.name || 'Admin'}! 👋</h2>
            <p className="text-blue-100 text-lg">
              Dashboard monitoring performa {schoolData?.students || 245} siswa Anda
            </p>
          </div>
          <Button size="lg" className="bg-white text-blue-600 hover:bg-blue-50">
            Download Laporan Lengkap
          </Button>
        </div>
      </Card>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card className="p-6 border-l-4 border-l-blue-500">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Total Siswa</p>
              <h3 className="text-3xl mb-2">{schoolData?.students || 245}</h3>
              <div className="flex items-center gap-1 text-sm text-blue-600">
                <Users className="w-4 h-4" />
                <span>198 aktif minggu ini</span>
              </div>
            </div>
            <div className="w-12 h-12 rounded-xl bg-blue-100 flex items-center justify-center">
              <Users className="w-6 h-6 text-blue-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6 border-l-4 border-l-green-500">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Rata-rata Skor</p>
              <h3 className="text-3xl mb-2">642</h3>
              <div className="flex items-center gap-1 text-sm text-green-600">
                <ArrowUp className="w-4 h-4" />
                <span>+8.2% dari bulan lalu</span>
              </div>
            </div>
            <div className="w-12 h-12 rounded-xl bg-green-100 flex items-center justify-center">
              <TrendingUp className="w-6 h-6 text-green-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6 border-l-4 border-l-purple-500">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Tryout Selesai</p>
              <h3 className="text-3xl mb-2">12</h3>
              <div className="flex items-center gap-1 text-sm text-purple-600">
                <CheckCircle2 className="w-4 h-4" />
                <span>2 ongoing</span>
              </div>
            </div>
            <div className="w-12 h-12 rounded-xl bg-purple-100 flex items-center justify-center">
              <Trophy className="w-6 h-6 text-purple-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6 border-l-4 border-l-orange-500">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Target PTN</p>
              <h3 className="text-3xl mb-2">85%</h3>
              <div className="flex items-center gap-1 text-sm text-orange-600">
                <Target className="w-4 h-4" />
                <span>Prediksi lolos</span>
              </div>
            </div>
            <div className="w-12 h-12 rounded-xl bg-orange-100 flex items-center justify-center">
              <Target className="w-6 h-6 text-orange-600" />
            </div>
          </div>
        </Card>
      </div>

      {/* Charts Row */}
      <div className="grid lg:grid-cols-3 gap-6">
        <Card className="lg:col-span-2 p-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg mb-1">Aktivitas Siswa Mingguan</h3>
              <p className="text-sm text-muted-foreground">Siswa aktif per hari</p>
            </div>
            <Button variant="outline" size="sm">7 Hari</Button>
          </div>
          
          <ResponsiveContainer width="100%" height={280}>
            <LineChart data={weeklyData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
              <XAxis dataKey="day" stroke="#888" />
              <YAxis stroke="#888" />
              <Tooltip 
                contentStyle={{ backgroundColor: '#fff', border: '1px solid #e5e7eb', borderRadius: '8px' }}
              />
              <Line
                key="line-active"
                name="Siswa Aktif"
                type="monotone"
                dataKey="active"
                stroke="#3b82f6"
                strokeWidth={3}
                dot={{ fill: '#3b82f6', r: 5 }}
                activeDot={{ r: 7 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </Card>

        <Card className="p-6">
          <h3 className="text-lg mb-1">Performa per Mata Uji</h3>
          <p className="text-sm text-muted-foreground mb-6">Rata-rata nilai</p>
          
          <div className="space-y-4">
            {subjectPerformance.map((item, idx) => (
              <div key={idx}>
                <div className="flex items-center justify-between mb-2 text-sm">
                  <span>{item.subject}</span>
                  <span>{item.avgScore}</span>
                </div>
                <div className="h-2.5 bg-slate-100 rounded-full overflow-hidden">
                  <div 
                    className="h-full rounded-full transition-all"
                    style={{ 
                      width: `${item.avgScore}%`,
                      backgroundColor: item.color
                    }}
                  ></div>
                </div>
              </div>
            ))}
          </div>

          <div className="mt-6 p-4 bg-gradient-to-br from-blue-50 to-cyan-50 rounded-lg border border-blue-100">
            <div className="flex items-center gap-2 mb-1">
              <Award className="w-4 h-4 text-blue-600" />
              <span className="text-sm">Overall Performance</span>
            </div>
            <p className="text-2xl text-blue-600">70.2%</p>
          </div>
        </Card>
      </div>

      {/* Top Students & Recent Activity */}
      <div className="grid lg:grid-cols-2 gap-6">
        <Card className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg mb-1">Top 5 Siswa Terbaik</h3>
              <p className="text-sm text-muted-foreground">Berdasarkan skor tryout terakhir</p>
            </div>
            <Button variant="outline" size="sm" onClick={() => onNavigate?.('students')}>Lihat Semua</Button>
          </div>

          <div className="space-y-3">
            {topStudents.map((student) => (
              <div key={student.rank} className="flex items-center justify-between p-3 bg-slate-50 rounded-lg hover:bg-slate-100 transition-colors">
                <div className="flex items-center gap-3">
                  <div className={`w-10 h-10 rounded-full flex items-center justify-center ${
                    student.rank === 1 ? 'bg-gradient-to-br from-yellow-400 to-orange-500 text-white' :
                    student.rank === 2 ? 'bg-gradient-to-br from-slate-300 to-slate-400 text-white' :
                    student.rank === 3 ? 'bg-gradient-to-br from-orange-300 to-orange-400 text-white' :
                    'bg-slate-200 text-slate-700'
                  }`}>
                    #{student.rank}
                  </div>
                  <div>
                    <div className="mb-1">{student.name}</div>
                    <div className="text-sm text-muted-foreground">Skor: {student.score}</div>
                  </div>
                </div>
                <Badge className="bg-green-100 text-green-700">
                  {student.improvement}
                </Badge>
              </div>
            ))}
          </div>
        </Card>

        <Card className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg mb-1">Aktivitas Terkini</h3>
              <p className="text-sm text-muted-foreground">Update terbaru dari siswa</p>
            </div>
          </div>

          <div className="space-y-4">
            {[
              { type: 'tryout', text: '15 siswa menyelesaikan Tryout Nasional #45', time: '2 jam lalu', icon: Trophy },
              { type: 'drill', text: '28 siswa aktif di Drilling Matematika', time: '4 jam lalu', icon: Target },
              { type: 'achievement', text: 'Ahmad Fauzi mencapai ranking 10 nasional', time: '6 jam lalu', icon: Award },
              { type: 'event', text: 'Event Tryout baru tersedia', time: '1 hari lalu', icon: Clock },
            ].map((activity, idx) => (
              <div key={idx} className="flex items-start gap-3 p-3 bg-slate-50 rounded-lg">
                <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center flex-shrink-0">
                  <activity.icon className="w-4 h-4 text-white" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm mb-1">{activity.text}</p>
                  <p className="text-xs text-muted-foreground">{activity.time}</p>
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
}
