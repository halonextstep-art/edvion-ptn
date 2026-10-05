import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { 
  Trophy, 
  Target, 
  Zap, 
  TrendingUp,
  Calendar,
  Award,
  Flame,
  Clock,
  Play,
  BookOpen
} from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, RadarChart, Radar, PolarGrid, PolarAngleAxis, PolarRadiusAxis } from 'recharts';

const scoreProgress = [
  { week: 'W1', score: 580 },
  { week: 'W2', score: 595 },
  { week: 'W3', score: 610 },
  { week: 'W4', score: 625 },
  { week: 'W5', score: 642 },
  { week: 'W6', score: 658 },
  { week: 'W7', score: 672 },
  { week: 'W8', score: 685 },
];

const subjectRadar = [
  { subject: 'TPS', value: 78 },
  { subject: 'Literasi ID', value: 72 },
  { subject: 'Literasi EN', value: 82 },
  { subject: 'Matematika', value: 68 },
  { subject: 'Penalaran', value: 75 },
];

const upcomingEvents = [
  { id: 1, name: 'Tryout Nasional #46', date: '25 Jan 2025', participants: 5234, type: 'Tryout' },
  { id: 2, name: 'Drilling Matematika', date: 'Setiap Hari', participants: 2134, type: 'Drilling' },
  { id: 3, name: 'Rasionalisasi Gel 2', date: '1 Feb 2025', participants: 3890, type: 'Rasionalisasi' },
];

const achievements = [
  { icon: '🏆', title: 'Top 100 Nasional', desc: 'Ranking 87 dari 50K+ siswa' },
  { icon: '🔥', title: '30 Day Streak', desc: 'Konsisten latihan 30 hari' },
  { icon: '⚡', title: 'Speed Master', desc: 'Selesaikan TO dalam waktu optimal' },
  { icon: '🎯', title: 'Perfect Score', desc: '100% di Literasi Inggris' },
];

interface StudentOverviewProps {
  userData?: any;
  onStartSession?: (session: { title: string; type: 'tryout' | 'drilling' | 'mini'; durationMinutes: number }) => void;
  onNavigate?: (tab: string) => void;
}

export default function StudentOverview({ userData, onStartSession, onNavigate }: StudentOverviewProps) {
  return (
    <div className="space-y-6">
      {/* Welcome Card */}
      <Card className="relative overflow-hidden p-6 bg-gradient-to-br from-purple-600 via-pink-600 to-orange-600 text-white">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLW9wYWNpdHk9IjAuMSIgc3Ryb2tlLXdpZHRoPSIxIi8+PC9wYXR0ZXJuPjwvZGVmcz48cmVjdCB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIiBmaWxsPSJ1cmwoI2dyaWQpIi8+PC9zdmc+')] opacity-30"></div>
        
        <div className="relative flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Flame className="w-6 h-6 text-yellow-300" />
              <span className="text-sm bg-white/20 px-3 py-1 rounded-full">30 hari streak 🔥</span>
            </div>
            <h2 className="text-3xl mb-2">Hai, {userData?.name || 'Pejuang PTN'}! 👋</h2>
            <p className="text-purple-100 text-lg">
              Terus semangat! Kamu sudah 15 poin lagi mencapai ranking 50 nasional
            </p>
          </div>
          <Button size="lg" className="bg-white text-purple-600 hover:bg-purple-50 gap-2" onClick={() => onStartSession?.({ title: 'Drilling Matematika — Level Sedang', type: 'drilling', durationMinutes: 20 })}>
            <Play className="w-5 h-5" />
            Mulai Drilling
          </Button>
        </div>
      </Card>

      {/* Quick Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className="p-5 border-2 border-purple-200 bg-gradient-to-br from-purple-50 to-white hover:shadow-lg transition-shadow">
          <div className="flex items-start justify-between mb-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center">
              <Trophy className="w-6 h-6 text-white" />
            </div>
            <Badge className="bg-green-100 text-green-700">+12</Badge>
          </div>
          <div className="text-3xl mb-1">685</div>
          <div className="text-sm text-muted-foreground">Skor Terakhir</div>
        </Card>

        <Card className="p-5 border-2 border-blue-200 bg-gradient-to-br from-blue-50 to-white hover:shadow-lg transition-shadow">
          <div className="flex items-start justify-between mb-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center">
              <TrendingUp className="w-6 h-6 text-white" />
            </div>
            <Badge className="bg-blue-100 text-blue-700">#87</Badge>
          </div>
          <div className="text-3xl mb-1">672</div>
          <div className="text-sm text-muted-foreground">Rata-rata Skor</div>
        </Card>

        <Card className="p-5 border-2 border-green-200 bg-gradient-to-br from-green-50 to-white hover:shadow-lg transition-shadow">
          <div className="flex items-start justify-between mb-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-green-500 to-emerald-500 flex items-center justify-center">
              <Target className="w-6 h-6 text-white" />
            </div>
            <Badge className="bg-green-100 text-green-700">85%</Badge>
          </div>
          <div className="text-3xl mb-1">UI</div>
          <div className="text-sm text-muted-foreground">Peluang PTN</div>
        </Card>

        <Card className="p-5 border-2 border-orange-200 bg-gradient-to-br from-orange-50 to-white hover:shadow-lg transition-shadow">
          <div className="flex items-start justify-between mb-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-orange-500 to-red-500 flex items-center justify-center">
              <Zap className="w-6 h-6 text-white" />
            </div>
            <Badge className="bg-orange-100 text-orange-700">+250</Badge>
          </div>
          <div className="text-3xl mb-1">2,450</div>
          <div className="text-sm text-muted-foreground">Reward Points</div>
        </Card>
      </div>

      {/* Charts */}
      <div className="grid lg:grid-cols-3 gap-6">
        <Card className="lg:col-span-2 p-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg mb-1">Progres Skor (8 Minggu Terakhir)</h3>
              <p className="text-sm text-muted-foreground">Peningkatan skor tryout kamu</p>
            </div>
            <div className="flex items-center gap-2">
              <Badge className="bg-green-100 text-green-700 gap-1">
                <TrendingUp className="w-3 h-3" />
                +105 poin
              </Badge>
            </div>
          </div>
          
          <svg style={{ position: 'absolute', width: 0, height: 0, overflow: 'hidden' }}>
            <defs>
              <linearGradient id="scoreGradient" x1="0" y1="0" x2="1" y2="0">
                <stop key="sg-0" offset="0%" stopColor="#8b5cf6" />
                <stop key="sg-1" offset="100%" stopColor="#ec4899" />
              </linearGradient>
            </defs>
          </svg>
          <ResponsiveContainer width="100%" height={280}>
            <LineChart data={scoreProgress}>
              <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
              <XAxis dataKey="week" stroke="#888" />
              <YAxis stroke="#888" domain={[550, 700]} />
              <Tooltip
                contentStyle={{ backgroundColor: '#fff', border: '1px solid #e5e7eb', borderRadius: '8px' }}
              />
              <Line
                key="line-score"
                type="monotone"
                dataKey="score"
                stroke="url(#scoreGradient)"
                strokeWidth={4}
                dot={{ fill: '#8b5cf6', r: 6 }}
                activeDot={{ r: 8 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </Card>

        <Card className="p-6">
          <h3 className="text-lg mb-1">Analisis Kemampuan</h3>
          <p className="text-sm text-muted-foreground mb-6">Radar chart per mata uji</p>
          
          <ResponsiveContainer width="100%" height={280}>
            <RadarChart data={subjectRadar}>
              <PolarGrid stroke="#e5e7eb" />
              <PolarAngleAxis dataKey="subject" tick={{ fontSize: 12 }} />
              <PolarRadiusAxis angle={90} domain={[0, 100]} />
              <Radar
                key="radar-score"
                name="Score"
                dataKey="value"
                stroke="#8b5cf6"
                fill="#8b5cf6"
                fillOpacity={0.6}
              />
              <Tooltip />
            </RadarChart>
          </ResponsiveContainer>
        </Card>
      </div>

      {/* Upcoming Events & Achievements */}
      <div className="grid lg:grid-cols-2 gap-6">
        <Card className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg mb-1">Event Mendatang</h3>
              <p className="text-sm text-muted-foreground">Tryout & drilling yang bisa diikuti</p>
            </div>
            <Button variant="outline" size="sm" onClick={() => onNavigate?.('drilling')}>Lihat Semua</Button>
          </div>

          <div className="space-y-3">
            {upcomingEvents.map((event) => (
              <div key={event.id} className="flex items-center justify-between p-4 bg-gradient-to-r from-slate-50 to-blue-50 rounded-lg border hover:border-blue-300 transition-colors">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center">
                    {event.type === 'Tryout' && <Trophy className="w-5 h-5 text-white" />}
                    {event.type === 'Drilling' && <Zap className="w-5 h-5 text-white" />}
                    {event.type === 'Rasionalisasi' && <Target className="w-5 h-5 text-white" />}
                  </div>
                  <div>
                    <div className="mb-1">{event.name}</div>
                    <div className="flex items-center gap-3 text-sm text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Calendar className="w-3.5 h-3.5" />
                        {event.date}
                      </span>
                      <Badge variant="outline" className="text-xs">{event.type}</Badge>
                    </div>
                  </div>
                </div>
                <Button size="sm" className="bg-gradient-to-r from-blue-600 to-cyan-600" onClick={() => onStartSession?.({ title: event.name, type: event.type === 'Tryout' ? 'tryout' : 'drilling', durationMinutes: event.type === 'Tryout' ? 30 : 15 })}>
                  <Play className="w-3.5 h-3.5 mr-1" />Mulai
                </Button>
              </div>
            ))}
          </div>
        </Card>

        <Card className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg mb-1">Achievements</h3>
              <p className="text-sm text-muted-foreground">Badge & pencapaian kamu</p>
            </div>
            <Button variant="outline" size="sm" onClick={() => onNavigate?.('leaderboard')}>Lihat Semua</Button>
          </div>

          <div className="grid grid-cols-2 gap-3">
            {achievements.map((achievement, idx) => (
              <div key={idx} className="p-4 border-2 rounded-lg hover:border-purple-300 transition-colors text-center">
                <div className="text-4xl mb-2">{achievement.icon}</div>
                <div className="text-sm mb-1">{achievement.title}</div>
                <div className="text-xs text-muted-foreground">{achievement.desc}</div>
              </div>
            ))}
          </div>

          <div className="mt-4 p-4 bg-gradient-to-r from-purple-50 to-pink-50 rounded-lg border border-purple-200">
            <div className="flex items-center gap-2 mb-2">
              <Award className="w-5 h-5 text-purple-600" />
              <span>Next Achievement</span>
            </div>
            <div className="text-sm text-muted-foreground mb-2">Selesaikan 5 tryout lagi untuk unlock "Marathon Runner" 🏃</div>
            <div className="h-2 bg-white rounded-full overflow-hidden">
              <div className="h-full bg-gradient-to-r from-purple-500 to-pink-500 rounded-full" style={{ width: '60%' }}></div>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
