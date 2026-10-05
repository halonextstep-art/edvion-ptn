import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { toast } from 'sonner';
import { 
  TrendingUp, 
  Trophy, 
  Target,
  Award,
  Calendar,
  Download,
  Share2,
  Flame,
  Zap,
  CheckCircle2,
  AlertTriangle
} from 'lucide-react';
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Area, AreaChart } from 'recharts';

const progressData = [
  { date: '1 Jan', score: 580, national: 565 },
  { date: '8 Jan', score: 595, national: 570 },
  { date: '15 Jan', score: 610, national: 575 },
  { date: '22 Jan', score: 625, national: 578 },
  { date: '29 Jan', score: 642, national: 582 },
  { date: '5 Feb', score: 658, national: 585 },
  { date: '12 Feb', score: 672, national: 588 },
  { date: '19 Feb', score: 685, national: 590 },
];

const subjectProgress = [
  { subject: 'TPS', jan: 65, feb: 78, improvement: 13 },
  { subject: 'Literasi ID', jan: 68, feb: 72, improvement: 4 },
  { subject: 'Literasi EN', jan: 75, feb: 82, improvement: 7 },
  { subject: 'Matematika', jan: 58, feb: 68, improvement: 10 },
];

const weeklyActivity = [
  { week: 'W1', tryout: 2, drilling: 5, hours: 8 },
  { week: 'W2', tryout: 1, drilling: 7, hours: 10 },
  { week: 'W3', tryout: 2, drilling: 6, hours: 9 },
  { week: 'W4', tryout: 3, drilling: 8, hours: 12 },
];

const milestones = [
  { id: 1, title: 'First Tryout', desc: 'Menyelesaikan tryout pertama', date: '15 Jan', achieved: true },
  { id: 2, title: 'Score 600+', desc: 'Mencapai skor di atas 600', date: '22 Jan', achieved: true },
  { id: 3, title: 'Top 100 Nasional', desc: 'Masuk ranking 100 besar', date: '5 Feb', achieved: true },
  { id: 4, title: '30 Day Streak', desc: 'Latihan konsisten 30 hari', date: '12 Feb', achieved: true },
  { id: 5, title: 'Score 700+', desc: 'Mencapai skor di atas 700', date: 'In Progress', achieved: false },
  { id: 6, title: 'Top 50 Nasional', desc: 'Masuk ranking 50 besar', date: 'Locked', achieved: false },
];

export default function StudentProgress() {
  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="p-6 bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 text-white relative overflow-hidden">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLW9wYWNpdHk9IjAuMSIgc3Ryb2tlLXdpZHRoPSIxIi8+PC9wYXR0ZXJuPjwvZGVmcz48cmVjdCB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIiBmaWxsPSJ1cmwoI2dyaWQpIi8+PC9zdmc+')] opacity-30"></div>
        
        <div className="relative flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 mb-2">
              <TrendingUp className="w-6 h-6 text-cyan-300" />
              <span className="text-lg">Progress Tracker</span>
            </div>
            <h2 className="text-3xl mb-2">Journey to PTN 🎯</h2>
            <p className="text-purple-100 text-lg">
              Track perkembangan dan capai target impian kamu
            </p>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" className="bg-white/10 border-white/30 text-white hover:bg-white/20 gap-2" onClick={() => toast.success('Link progress berhasil disalin!', { description: 'Bagikan ke teman-temanmu untuk motivasi bersama 🔥' })}>
              <Share2 className="w-4 h-4" />
              Share Progress
            </Button>
            <Button className="bg-white text-purple-600 hover:bg-purple-50 gap-2" onClick={() => toast.info('Menyiapkan laporan...', { description: 'Laporan PDF akan segera diunduh' })}>
              <Download className="w-4 h-4" />
              Download Report
            </Button>
          </div>
        </div>
      </Card>

      {/* Achievement Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className="p-5 bg-gradient-to-br from-yellow-50 to-orange-50 border-2 border-yellow-200">
          <div className="flex items-center justify-between mb-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-yellow-400 to-orange-500 flex items-center justify-center">
              <Flame className="w-6 h-6 text-white" />
            </div>
            <Trophy className="w-8 h-8 text-yellow-600 opacity-20" />
          </div>
          <div className="text-3xl mb-1">30</div>
          <div className="text-sm text-muted-foreground">Day Streak</div>
        </Card>

        <Card className="p-5 bg-gradient-to-br from-purple-50 to-pink-50 border-2 border-purple-200">
          <div className="flex items-center justify-between mb-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center">
              <Trophy className="w-6 h-6 text-white" />
            </div>
            <Target className="w-8 h-8 text-purple-600 opacity-20" />
          </div>
          <div className="text-3xl mb-1">8</div>
          <div className="text-sm text-muted-foreground">Tryouts Done</div>
        </Card>

        <Card className="p-5 bg-gradient-to-br from-blue-50 to-cyan-50 border-2 border-blue-200">
          <div className="flex items-center justify-between mb-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center">
              <Zap className="w-6 h-6 text-white" />
            </div>
            <Award className="w-8 h-8 text-blue-600 opacity-20" />
          </div>
          <div className="text-3xl mb-1">245</div>
          <div className="text-sm text-muted-foreground">Questions Solved</div>
        </Card>

        <Card className="p-5 bg-gradient-to-br from-green-50 to-emerald-50 border-2 border-green-200">
          <div className="flex items-center justify-between mb-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-green-500 to-emerald-500 flex items-center justify-center">
              <TrendingUp className="w-6 h-6 text-white" />
            </div>
            <CheckCircle2 className="w-8 h-8 text-green-600 opacity-20" />
          </div>
          <div className="text-3xl mb-1">+105</div>
          <div className="text-sm text-muted-foreground">Score Improvement</div>
        </Card>
      </div>

      {/* Score Progress Chart */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-xl mb-1">Perkembangan Skor</h3>
            <p className="text-sm text-muted-foreground">
              Perbandingan skor kamu vs rata-rata nasional
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Badge className="bg-purple-100 text-purple-700">
              <TrendingUp className="w-3 h-3 mr-1" />
              +18% growth
            </Badge>
          </div>
        </div>

        <svg style={{ position: 'absolute', width: 0, height: 0, overflow: 'hidden' }}>
          <defs>
            <linearGradient id="colorScore" x1="0" y1="0" x2="0" y2="1">
              <stop key="cs-0" offset="5%" stopColor="#8b5cf6" stopOpacity={0.8} />
              <stop key="cs-1" offset="95%" stopColor="#8b5cf6" stopOpacity={0} />
            </linearGradient>
            <linearGradient id="colorNational" x1="0" y1="0" x2="0" y2="1">
              <stop key="cn-0" offset="5%" stopColor="#94a3b8" stopOpacity={0.4} />
              <stop key="cn-1" offset="95%" stopColor="#94a3b8" stopOpacity={0} />
            </linearGradient>
          </defs>
        </svg>
        <ResponsiveContainer width="100%" height={320}>
          <AreaChart data={progressData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
            <XAxis dataKey="date" stroke="#888" />
            <YAxis stroke="#888" domain={[550, 700]} />
            <Tooltip
              contentStyle={{ backgroundColor: '#fff', border: '1px solid #e5e7eb', borderRadius: '8px' }}
            />
            <Area
              key="area-score"
              type="monotone"
              dataKey="score"
              stroke="#8b5cf6"
              strokeWidth={3}
              fillOpacity={1}
              fill="url(#colorScore)"
              name="Skor Kamu"
            />
            <Area
              key="area-national"
              type="monotone"
              dataKey="national"
              stroke="#94a3b8"
              strokeWidth={2}
              strokeDasharray="5 5"
              fillOpacity={1}
              fill="url(#colorNational)"
              name="Rata-rata Nasional"
            />
          </AreaChart>
        </ResponsiveContainer>

        <div className="flex items-center justify-center gap-6 mt-4">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full bg-purple-500"></div>
            <span className="text-sm">Skor Kamu</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full bg-slate-400"></div>
            <span className="text-sm">Rata-rata Nasional</span>
          </div>
        </div>
      </Card>

      {/* Subject Improvement & Activity */}
      <div className="grid lg:grid-cols-2 gap-6">
        <Card className="p-6">
          <h3 className="text-xl mb-1">Peningkatan Per Mata Uji</h3>
          <p className="text-sm text-muted-foreground mb-6">Januari vs Februari</p>

          <div className="space-y-4">
            {subjectProgress.map((subject, idx) => (
              <div key={idx} className="p-4 border rounded-lg hover:border-purple-300 transition-colors">
                <div className="flex items-center justify-between mb-3">
                  <span>{subject.subject}</span>
                  <Badge className={subject.improvement >= 10 ? 'bg-green-100 text-green-700' : 'bg-blue-100 text-blue-700'}>
                    +{subject.improvement}
                  </Badge>
                </div>
                <div className="flex items-center gap-3">
                  <div className="flex-1">
                    <div className="text-xs text-muted-foreground mb-1">Januari</div>
                    <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                      <div 
                        className="h-full bg-slate-400 rounded-full"
                        style={{ width: `${subject.jan}%` }}
                      ></div>
                    </div>
                  </div>
                  <div className="text-sm text-muted-foreground">{subject.jan}%</div>
                </div>
                <div className="flex items-center gap-3 mt-2">
                  <div className="flex-1">
                    <div className="text-xs text-muted-foreground mb-1">Februari</div>
                    <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                      <div 
                        className="h-full bg-gradient-to-r from-purple-500 to-pink-500 rounded-full"
                        style={{ width: `${subject.feb}%` }}
                      ></div>
                    </div>
                  </div>
                  <div className="text-sm">{subject.feb}%</div>
                </div>
              </div>
            ))}
          </div>
        </Card>

        <Card className="p-6">
          <h3 className="text-xl mb-1">Aktivitas Mingguan</h3>
          <p className="text-sm text-muted-foreground mb-6">Tryout & drilling per minggu</p>

          <ResponsiveContainer width="100%" height={280}>
            <BarChart data={weeklyActivity}>
              <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
              <XAxis dataKey="week" stroke="#888" />
              <YAxis stroke="#888" />
              <Tooltip 
                contentStyle={{ backgroundColor: '#fff', border: '1px solid #e5e7eb', borderRadius: '8px' }}
              />
              <Bar key="bar-tryout" dataKey="tryout" fill="#8b5cf6" radius={[8, 8, 0, 0]} name="Tryout" />
              <Bar key="bar-drilling" dataKey="drilling" fill="#ec4899" radius={[8, 8, 0, 0]} name="Drilling" />
            </BarChart>
          </ResponsiveContainer>

          <div className="grid grid-cols-3 gap-3 mt-4">
            <div className="p-3 bg-purple-50 rounded-lg text-center">
              <div className="text-2xl mb-1">8</div>
              <div className="text-xs text-muted-foreground">Tryout</div>
            </div>
            <div className="p-3 bg-pink-50 rounded-lg text-center">
              <div className="text-2xl mb-1">26</div>
              <div className="text-xs text-muted-foreground">Drilling</div>
            </div>
            <div className="p-3 bg-blue-50 rounded-lg text-center">
              <div className="text-2xl mb-1">39h</div>
              <div className="text-xs text-muted-foreground">Total Hours</div>
            </div>
          </div>
        </Card>
      </div>

      {/* Milestones */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-xl mb-1">Milestones & Achievements</h3>
            <p className="text-sm text-muted-foreground">
              Pencapaian yang sudah kamu raih
            </p>
          </div>
          <Badge className="bg-purple-100 text-purple-700">
            4 / 6 Completed
          </Badge>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
          {milestones.map((milestone) => (
            <div 
              key={milestone.id}
              className={`p-4 border-2 rounded-lg transition-all ${
                milestone.achieved 
                  ? 'border-green-200 bg-gradient-to-br from-green-50 to-emerald-50' 
                  : 'border-slate-200 bg-slate-50 opacity-60'
              }`}
            >
              <div className="flex items-start justify-between mb-3">
                <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                  milestone.achieved 
                    ? 'bg-gradient-to-br from-green-500 to-emerald-500' 
                    : 'bg-slate-300'
                }`}>
                  {milestone.achieved ? (
                    <CheckCircle2 className="w-5 h-5 text-white" />
                  ) : (
                    <Calendar className="w-5 h-5 text-white" />
                  )}
                </div>
                <Badge variant="outline" className="text-xs">
                  {milestone.date}
                </Badge>
              </div>
              <h4 className="mb-1">{milestone.title}</h4>
              <p className="text-sm text-muted-foreground">{milestone.desc}</p>
            </div>
          ))}
        </div>
      </Card>

      {/* Insights */}
      <Card className="p-6 bg-gradient-to-br from-blue-50 to-purple-50 border-blue-200">
        <div className="flex items-start gap-4">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-600 to-purple-600 flex items-center justify-center flex-shrink-0">
            <TrendingUp className="w-6 h-6 text-white" />
          </div>
          <div>
            <h4 className="mb-3">📊 Insight & Rekomendasi</h4>
            <ul className="space-y-2 text-sm">
              <li className="flex items-start gap-2">
                <CheckCircle2 className="w-4 h-4 mt-0.5 text-green-600 flex-shrink-0" />
                <span><strong>Great Progress!</strong> Skor kamu meningkat 105 poin dalam 8 minggu (rata-rata +13 poin/minggu)</span>
              </li>
              <li className="flex items-start gap-2">
                <AlertTriangle className="w-4 h-4 mt-0.5 text-orange-600 flex-shrink-0" />
                <span><strong>Fokus Area:</strong> Matematika masih di bawah target. Tambah drilling 2-3x seminggu untuk boost performance</span>
              </li>
              <li className="flex items-start gap-2">
                <CheckCircle2 className="w-4 h-4 mt-0.5 text-green-600 flex-shrink-0" />
                <span><strong>Konsistensi Excellent:</strong> 30 hari streak menunjukkan dedikasi tinggi. Keep it up!</span>
              </li>
            </ul>
          </div>
        </div>
      </Card>
    </div>
  );
}
