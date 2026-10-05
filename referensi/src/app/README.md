# GASPOLPTN - Sistem Persiapan SNBT End-to-End

Platform terlengkap untuk persiapan SNBT dengan fitur Drilling Tryout dan Rasionalisasi PTN berbasis AI.

## 🎯 Fitur Utama

### 👨‍💼 Admin Pusat (GASPOL HQ)
- **Dashboard Overview**: Monitoring real-time aktivitas nasional
- **Bank Soal**: Kelola 10K+ soal dengan editor canggih (LaTeX support, stimulus, multi-jenis)
- **Event Management**: Atur Tryout, Rasionalisasi, dan Drilling Events
- **Partner Management**: Kelola 500+ sekolah mitra B2B
- **Analytics**: Insight nasional dan performa sistem

### 🏫 Portal Sekolah Mitra
- **Overview**: Dashboard performa 245 siswa dengan charts interaktif
- **Student Management**: Monitor progress individual dan kolektif
- **Analytics**: Radar chart, heatmap topik, distribusi target PTN
- **Laporan**: Export PDF dan Excel untuk guru pembimbing
- **Note**: Pengelolaan paket soal TIDAK diserahkan ke sekolah (dikelola Admin Pusat)

### 🎓 Portal Siswa
- **Dashboard**: Progress tracking dengan gamifikasi (badges, streak, points)
- **Drilling Zone**: 
  - Tryout Resmi (event berjadwal)
  - Latihan Harian (AI adaptif)
  - Latihan Per Topik
  - Speed Challenge
- **Rasionalisasi SNBT**: Prediksi peluang PTN dengan AI
- **Progress Tracker**: Visualisasi journey dengan milestone

## 🔐 Demo Login Credentials

### Siswa
```
Email: siswa@student.com
Password: siswa123
```
**Akses**: Drilling tryout, rasionalisasi PTN, progress tracking

### Sekolah
```
Email: sekolah@sman1.sch.id
Password: sekolah123
```
**Akses**: Monitoring siswa, analytics kolektif, laporan

### Admin
```
Email: admin@gaspolptn.com
Password: admin123
```
**Akses**: Bank soal, event management, partner management, analytics nasional

## 🚀 Cara Menggunakan

1. Buka aplikasi
2. Pilih role (Siswa / Sekolah / Admin)
3. Masukkan credentials demo atau klik "Quick Demo Login"
4. Explore dashboard sesuai role

## 🎨 Teknologi

- **Frontend**: React + TypeScript
- **Styling**: Tailwind CSS v4.0
- **UI Components**: Shadcn/ui
- **Charts**: Recharts
- **Icons**: Lucide React

## 📊 Statistik Platform

- **50K+** Siswa Aktif
- **500+** Sekolah Mitra
- **10K+** Soal Berkualitas
- **95%** Tingkat Kepuasan

## 🏗️ Struktur Fitur

### Question Bank System
- Support 3 jenis soal: Pilihan Ganda, Majemuk Kompleks, Isian Singkat
- LaTeX equation support untuk matematika
- Stimulus multi-soal untuk literasi & penalaran
- Auto-validation dan quality check
- Real-time preview

### AI Engine
- Adaptive drilling algorithm
- Predictive admission analysis
- Personalized recommendations
- Performance analytics

### Gamification
- Reward points system
- Achievement badges
- National & school leaderboards
- Daily streak tracking

## 📝 Important Notes

1. **Pengelolaan Soal**: Hanya Admin Pusat yang bisa mengelola bank soal dan paket soal
2. **Sekolah**: Fokus pada monitoring dan analisis performa siswa
3. **Data**: Semua data adalah demo/mock data untuk keperluan demonstrasi

## 🎯 Role & Permissions

| Fitur | Admin | Sekolah | Siswa |
|-------|-------|---------|-------|
| Bank Soal | ✅ Kelola | ❌ | ❌ |
| Event Management | ✅ Kelola | 👁️ View | 👁️ View |
| Monitor Siswa | ✅ Semua | ✅ Sekolah | ❌ |
| Drilling/Tryout | ❌ | ❌ | ✅ |
| Rasionalisasi PTN | ✅ View All | ✅ Sekolah | ✅ |
| Analytics | ✅ Nasional | ✅ Sekolah | ✅ Personal |

---

**© 2025 GASPOLPTN** - Platform Persiapan SNBT Terlengkap dengan AI Technology
