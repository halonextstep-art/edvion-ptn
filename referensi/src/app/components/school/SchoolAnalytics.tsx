import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Target, TrendingUp, Users, Award } from 'lucide-react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, RadarChart, Radar, PolarGrid, PolarAngleAxis, PolarRadiusAxis } from 'recharts';

const topicMastery = [
  { topic: 'Aljabar', mastery: 72, difficulty: 'Sedang' },
  { topic: 'Geometri', mastery: 68, difficulty: 'Sulit' },
  { topic: 'Statistika', mastery: 78, difficulty: 'Mudah' },
  { topic: 'Logika', mastery: 65, difficulty: 'Sulit' },
  { topic: 'Literasi Teks', mastery: 75, difficulty: 'Sedang' },
  { topic: 'Reading Comp', mastery: 70, difficulty: 'Sedang' },
];

const radarData = [
  { subject: 'TPS', value: 72 },
  { subject: 'Literasi ID', value: 68 },
  { subject: 'Literasi EN', value: 75 },
  { subject: 'Matematika', value: 64 },
  { subject: 'Penalaran', value: 70 },
];

const ptnTargets = [
  { ptn: 'UI', count: 45, avgChance: 78, color: '#6366f1' },
  { ptn: 'ITB', count: 38, avgChance: 72, color: '#8b5cf6' },
  { ptn: 'UGM', count: 52, avgChance: 80, color: '#ec4899' },
  { ptn: 'UNPAD', count: 28, avgChance: 85, color: '#f59e0b' },
  { ptn: 'UNDIP', count: 22, avgChance: 82, color: '#10b981' },
  { ptn: 'UNAIR', count: 35, avgChance: 76, color: '#3b82f6' },
];

export default function SchoolAnalytics() {
  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="p-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div>
            <h2 className="text-2xl mb-1">Analytics & Insights</h2>
            <p className="text-sm text-muted-foreground">
              Analisis mendalam performa siswa dan prediksi PTN
            </p>
          </div>
          
          <div className="flex gap-2">
            <Button variant="outline">Export PDF</Button>
            <Button className="bg-gradient-to-r from-blue-600 to-cyan-600">
              Generate Report
            </Button>
          </div>
        </div>
      </Card>

      {/* Info Card - Pengelolaan Soal */}
      <Card className="p-6 bg-gradient-to-br from-blue-50 to-cyan-50 border-blue-200">
        <div className="flex items-start gap-4">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-600 to-cyan-600 flex items-center justify-center flex-shrink-0">
            <TrendingUp className="w-6 h-6 text-white" />
          </div>
          <div>
            <h4 className="mb-2">ℹ️ Informasi Penting</h4>
            <p className="text-sm text-muted-foreground">
              Pengelolaan paket soal dan bank soal sepenuhnya ditangani oleh <strong>Admin Pusat GASPOLPTN</strong>. 
              Sekolah fokus pada monitoring dan analisis performa siswa untuk hasil optimal.
            </p>
          </div>
        </div>
      </Card>

      {/* Performance Overview */}
      <div className="grid lg:grid-cols-2 gap-6">
        <Card className="p-6">
          <h3 className="text-lg mb-1">Radar Performa Mata Uji</h3>
          <p className="text-sm text-muted-foreground mb-6">Rata-rata penguasaan per kategori</p>
          
          <ResponsiveContainer width="100%" height={320}>
            <RadarChart data={radarData}>
              <PolarGrid stroke="#e5e7eb" />
              <PolarAngleAxis dataKey="subject" />
              <PolarRadiusAxis angle={90} domain={[0, 100]} />
              <Radar
                key="radar-school"
                name="Performa Sekolah"
                dataKey="value"
                stroke="#3b82f6"
                fill="#3b82f6"
                fillOpacity={0.6}
              />
              <Tooltip />
            </RadarChart>
          </ResponsiveContainer>

          <div className="flex items-center justify-center gap-4 mt-4">
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 rounded-full bg-blue-500"></div>
              <span className="text-sm">Performa Sekolah</span>
            </div>
          </div>
        </Card>

        <Card className="p-6">
          <h3 className="text-lg mb-1">Penguasaan Topik</h3>
          <p className="text-sm text-muted-foreground mb-6">Heatmap performa per materi</p>
          
          <div className="space-y-3">
            {topicMastery.map((topic, idx) => (
              <div key={idx} className="p-3 border rounded-lg hover:border-blue-300 transition-colors">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <span>{topic.topic}</span>
                    <Badge variant="outline" className={
                      topic.difficulty === 'Mudah' ? 'border-green-500 text-green-700' :
                      topic.difficulty === 'Sedang' ? 'border-orange-500 text-orange-700' :
                      'border-red-500 text-red-700'
                    }>
                      {topic.difficulty}
                    </Badge>
                  </div>
                  <span className="text-sm">{topic.mastery}%</span>
                </div>
                <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                  <div 
                    className={`h-full rounded-full ${
                      topic.mastery >= 75 ? 'bg-gradient-to-r from-green-500 to-emerald-500' :
                      topic.mastery >= 60 ? 'bg-gradient-to-r from-orange-500 to-yellow-500' :
                      'bg-gradient-to-r from-red-500 to-pink-500'
                    }`}
                    style={{ width: `${topic.mastery}%` }}
                  ></div>
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>

      {/* PTN Target Distribution */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-lg mb-1">Distribusi Target PTN</h3>
            <p className="text-sm text-muted-foreground">Pilihan PTN siswa dan prediksi peluang</p>
          </div>
          <Button variant="outline" size="sm">Filter PTN</Button>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
          {ptnTargets.map((ptn, idx) => (
            <Card key={idx} className="p-5 border-2 hover:border-blue-300 transition-colors">
              <div className="flex items-start justify-between mb-4">
                <div>
                  <h4 className="text-lg mb-1">{ptn.ptn}</h4>
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Users className="w-4 h-4" />
                    <span>{ptn.count} siswa</span>
                  </div>
                </div>
                <div className="w-10 h-10 rounded-lg flex items-center justify-center" style={{ backgroundColor: `${ptn.color}20` }}>
                  <Target className="w-5 h-5" style={{ color: ptn.color }} />
                </div>
              </div>

              <div className="mb-3">
                <div className="flex items-center justify-between text-sm mb-1.5">
                  <span className="text-muted-foreground">Rata-rata Peluang</span>
                  <span style={{ color: ptn.color }}>{ptn.avgChance}%</span>
                </div>
                <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                  <div 
                    className="h-full rounded-full"
                    style={{ 
                      width: `${ptn.avgChance}%`,
                      background: `linear-gradient(to right, ${ptn.color}, ${ptn.color}cc)`
                    }}
                  ></div>
                </div>
              </div>

              <Badge 
                className="w-full justify-center"
                style={{ 
                  backgroundColor: `${ptn.color}15`,
                  color: ptn.color,
                  border: `1px solid ${ptn.color}40`
                }}
              >
                {ptn.avgChance >= 80 ? 'Peluang Tinggi' : 
                 ptn.avgChance >= 65 ? 'Peluang Sedang' : 
                 'Perlu Peningkatan'}
              </Badge>
            </Card>
          ))}
        </div>
      </Card>

      {/* Insights & Recommendations */}
      <div className="grid lg:grid-cols-2 gap-6">
        <Card className="p-6 bg-gradient-to-br from-blue-50 to-cyan-50 border-blue-200">
          <div className="flex items-start gap-3 mb-4">
            <div className="w-10 h-10 rounded-lg bg-blue-600 flex items-center justify-center flex-shrink-0">
              <TrendingUp className="w-5 h-5 text-white" />
            </div>
            <div>
              <h3 className="mb-2">Strengths (Kekuatan)</h3>
              <p className="text-sm text-muted-foreground">Area yang sudah dikuasai dengan baik</p>
            </div>
          </div>
          <ul className="space-y-2">
            <li className="flex items-start gap-2 text-sm">
              <div className="w-1.5 h-1.5 rounded-full bg-green-500 mt-1.5 flex-shrink-0"></div>
              <span><strong>Statistika</strong> - 78% mastery, sangat baik untuk soal TPS Kuantitatif</span>
            </li>
            <li className="flex items-start gap-2 text-sm">
              <div className="w-1.5 h-1.5 rounded-full bg-green-500 mt-1.5 flex-shrink-0"></div>
              <span><strong>Literasi Inggris</strong> - 75% mastery, di atas rata-rata nasional</span>
            </li>
            <li className="flex items-start gap-2 text-sm">
              <div className="w-1.5 h-1.5 rounded-full bg-green-500 mt-1.5 flex-shrink-0"></div>
              <span><strong>Konsistensi</strong> - 85% siswa aktif mengikuti drilling rutin</span>
            </li>
          </ul>
        </Card>

        <Card className="p-6 bg-gradient-to-br from-orange-50 to-red-50 border-orange-200">
          <div className="flex items-start gap-3 mb-4">
            <div className="w-10 h-10 rounded-lg bg-orange-600 flex items-center justify-center flex-shrink-0">
              <Award className="w-5 h-5 text-white" />
            </div>
            <div>
              <h3 className="mb-2">Improvements (Area Peningkatan)</h3>
              <p className="text-sm text-muted-foreground">Fokus untuk ditingkatkan</p>
            </div>
          </div>
          <ul className="space-y-2">
            <li className="flex items-start gap-2 text-sm">
              <div className="w-1.5 h-1.5 rounded-full bg-red-500 mt-1.5 flex-shrink-0"></div>
              <span><strong>Penalaran Matematika</strong> - 64% mastery, perlu drilling intensif</span>
            </li>
            <li className="flex items-start gap-2 text-sm">
              <div className="w-1.5 h-1.5 rounded-full bg-red-500 mt-1.5 flex-shrink-0"></div>
              <span><strong>Logika & Penalaran</strong> - 65% mastery, topik tersulit</span>
            </li>
            <li className="flex items-start gap-2 text-sm">
              <div className="w-1.5 h-1.5 rounded-full bg-orange-500 mt-1.5 flex-shrink-0"></div>
              <span><strong>Geometri</strong> - 68% mastery, banyak siswa struggle di soal aplikasi</span>
            </li>
          </ul>
        </Card>
      </div>
    </div>
  );
}