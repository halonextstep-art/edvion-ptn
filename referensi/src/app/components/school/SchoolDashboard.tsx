import { useState } from 'react';
import { Button } from '../ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import {
  LogOut, LayoutDashboard, Users, TrendingUp, FileText, School, Target, ClipboardList
} from 'lucide-react';
import SchoolOverview from './SchoolOverview';
import StudentManagement from './StudentManagement';
import SchoolAnalytics from './SchoolAnalytics';
import SchoolReports from './SchoolReports';
import SNBPRasionalisasi from './SNBPRasionalisasi';
import RekapSNBT from './RekapSNBT';
import NotificationBell from '../shared/NotificationBell';

interface SchoolDashboardProps {
  onLogout: () => void;
  userData?: any;
}

export default function SchoolDashboard({ onLogout, userData }: SchoolDashboardProps) {
  const [activeTab, setActiveTab] = useState('overview');

  return (
    <div className="min-h-screen bg-slate-50">
      <div className="bg-white border-b sticky top-0 z-50">
        <div className="max-w-[1800px] mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-600 to-cyan-600 flex items-center justify-center">
                <School className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-xl font-semibold">{userData?.name || 'Sekolah'}</h1>
                <p className="text-sm text-muted-foreground">Portal Sekolah Mitra</p>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <NotificationBell role="school" />
              <div className="w-px h-8 bg-slate-200" />
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-full bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center">
                  <span className="text-white text-sm font-bold">S</span>
                </div>
                <div className="hidden sm:block">
                  <div className="text-sm font-medium">{userData?.name || 'Admin Sekolah'}</div>
                  <div className="text-xs text-muted-foreground">{userData?.students || 0} siswa</div>
                </div>
              </div>
              <Button variant="ghost" size="icon" onClick={onLogout}>
                <LogOut className="w-5 h-5" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-[1800px] mx-auto px-6 py-8">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
          <TabsList className="inline-flex h-auto p-1 bg-white border shadow-sm flex-wrap">
            <TabsTrigger value="overview" className="gap-2">
              <LayoutDashboard className="w-4 h-4" />
              <span className="hidden sm:inline">Overview</span>
            </TabsTrigger>
            <TabsTrigger value="students" className="gap-2">
              <Users className="w-4 h-4" />
              <span className="hidden sm:inline">Siswa</span>
            </TabsTrigger>
            <TabsTrigger value="snbp" className="gap-2">
              <Target className="w-4 h-4" />
              <span className="hidden sm:inline">Rasionalisasi SNBP</span>
            </TabsTrigger>
            <TabsTrigger value="snbt" className="gap-2">
              <ClipboardList className="w-4 h-4" />
              <span className="hidden sm:inline">Rekap SNBT</span>
            </TabsTrigger>
            <TabsTrigger value="analytics" className="gap-2">
              <TrendingUp className="w-4 h-4" />
              <span className="hidden sm:inline">Analytics</span>
            </TabsTrigger>
            <TabsTrigger value="reports" className="gap-2">
              <FileText className="w-4 h-4" />
              <span className="hidden sm:inline">Laporan</span>
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview"><SchoolOverview schoolData={userData} onNavigate={setActiveTab} /></TabsContent>
          <TabsContent value="students"><StudentManagement /></TabsContent>
          <TabsContent value="snbp"><SNBPRasionalisasi /></TabsContent>
          <TabsContent value="snbt"><RekapSNBT /></TabsContent>
          <TabsContent value="analytics"><SchoolAnalytics /></TabsContent>
          <TabsContent value="reports"><SchoolReports /></TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
