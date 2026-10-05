<script setup lang="ts">
// Konten — menggabungkan 5 menu Admin yang tadinya sejajar (Taksonomi, Bank Soal, Set Soal,
// Sesi Tryout & Drilling, Paket) jadi satu tab dengan 2 lapis sub-navigasi. Alasannya: dari
// kelimanya, cuma Bank Soal dan Paket (lewat wizard "Buat Paket + Subtes") yang benar-benar
// jadi alur kerja harian — Taksonomi cuma disentuh sesekali saat setup, dan Set Soal/Sesi di
// alur normal sudah dibuat & dikelola OTOMATIS oleh wizard Paket per subtes (lihat
// PackageWizard.vue materializeSubtests()/loadSoal()). Menaruh keduanya sejajar dengan Bank
// Soal & Paket di navigasi utama bikin admin baru mengira ada 5 hal terpisah yang harus
// dipelajari, padahal 3 di antaranya adalah infrastruktur pendukung yang jarang dibuka
// langsung. Jadi di sini: "Bank Soal" dan "Paket & Subtes" adalah menu utama, sisanya
// dikelompokkan ke "Lanjutan" untuk kasus khusus (reuse Set Soal lintas sesi, edit detail sesi
// yang tidak tercakup wizard, atau menambah kategori/mata uji baru).
import { FileText, Package, SlidersHorizontal, FolderTree, ListChecks, Zap, GraduationCap } from 'lucide-vue-next'

const subTab = ref<'soal' | 'paket' | 'lanjutan'>('soal')
const advancedTab = ref<'taksonomi' | 'set-soal' | 'sesi' | 'ptn'>('taksonomi')

// Deep-link dari kartu subtes di PackageWizard ("Kelola Sesi ini" / "Kelola Set Soal ini") —
// begitu diklik, langsung lompat ke sub-tab Lanjutan yang sesuai dan fokus ke item itu,
// alih-alih admin harus tahu sendiri menu itu ada lalu mencari itemnya secara manual.
const focusSessionId = ref<string | null>(null)
const focusSetId = ref<string | null>(null)

function goToSession(sessionId: string) {
  focusSessionId.value = sessionId
  advancedTab.value = 'sesi'
  subTab.value = 'lanjutan'
}
function goToSet(setId: string) {
  focusSetId.value = setId
  advancedTab.value = 'set-soal'
  subTab.value = 'lanjutan'
}
</script>

<template>
  <div class="space-y-5">
    <Tabs v-model="subTab" class="space-y-5">
      <TabsList class="inline-flex h-auto p-1 bg-white border shadow-sm">
        <TabsTrigger value="soal" class="gap-2"><FileText class="w-4 h-4" />Bank Soal</TabsTrigger>
        <TabsTrigger value="paket" class="gap-2"><Package class="w-4 h-4" />Paket &amp; Subtes</TabsTrigger>
        <TabsTrigger value="lanjutan" class="gap-2"><SlidersHorizontal class="w-4 h-4" />Lanjutan</TabsTrigger>
      </TabsList>

      <TabsContent value="soal"><QuestionBankManager /></TabsContent>
      <TabsContent value="paket"><PackageManager @go-to-session="goToSession" @go-to-set="goToSet" /></TabsContent>

      <TabsContent value="lanjutan" class="space-y-5">
        <p class="text-xs text-muted-foreground flex items-start gap-1.5 bg-slate-50 border rounded-lg p-3">
          <SlidersHorizontal class="w-3.5 h-3.5 shrink-0 mt-0.5" />
          Alat pendukung yang jarang dipakai langsung — Set Soal dan Sesi biasanya sudah dibuat &amp; dikelola otomatis lewat wizard "Buat Paket + Subtes" di tab Paket. Buka di sini untuk kasus khusus: pakai ulang satu Set Soal di beberapa sesi, ubah detail sesi yang tidak ada di wizard, atau menambah kategori/mata uji baru.
        </p>
        <Tabs v-model="advancedTab" class="space-y-5">
          <TabsList class="inline-flex h-auto p-1 bg-slate-100">
            <TabsTrigger value="taksonomi" class="gap-2"><FolderTree class="w-4 h-4" />Taksonomi</TabsTrigger>
            <TabsTrigger value="set-soal" class="gap-2"><ListChecks class="w-4 h-4" />Set Soal</TabsTrigger>
            <TabsTrigger value="sesi" class="gap-2"><Zap class="w-4 h-4" />Sesi Tryout &amp; Drilling</TabsTrigger>
            <TabsTrigger value="ptn" class="gap-2"><GraduationCap class="w-4 h-4" />Katalog PTN</TabsTrigger>
          </TabsList>

          <TabsContent value="taksonomi"><TaxonomyManager /></TabsContent>
          <TabsContent value="set-soal"><QuestionSetManager :focus-set-id="focusSetId" /></TabsContent>
          <TabsContent value="sesi"><SessionManager :focus-session-id="focusSessionId" /></TabsContent>
          <TabsContent value="ptn"><PtnCatalogManager /></TabsContent>
        </Tabs>
      </TabsContent>
    </Tabs>
  </div>
</template>
