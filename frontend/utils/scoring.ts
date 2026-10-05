// Real platform scoring ceiling — sama persis dengan `PLATFORM_SCALE_MAX` di backend
// (lihat rationalization_service.rs). Dipusatkan di sini supaya nilai "1000" yang dipakai
// untuk menghitung persentase skor di berbagai chart/progress-bar tidak lagi ditulis
// berulang sebagai angka ajaib di banyak file.
export const PLATFORM_MAX_SCORE = 1000
