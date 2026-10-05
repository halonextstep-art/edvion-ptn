import { useState } from 'react';
import { Button } from '../ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import {
  LogOut, LayoutDashboard, FileText, Calendar,
  DollarSign, TrendingUp, School, UserCog, Sparkles, Tag, Package
} from 'lucide-react';
import AdminOverview from './AdminOverview';
import QuestionBank from './QuestionBank';
import EventManagement from './EventManagement';
import PartnerManagement from './PartnerManagement';
import AdminAnalytics from './AdminAnalytics';
import AdminFinance from './AdminFinance';
import AdminUserManagement from './AdminUserManagement';
import AdminGamification from './AdminGamification';
import AdminPackageManagement from './AdminPackageManagement';
import AdminVoucher from './AdminVoucher';
import NotificationBell from '../shared/NotificationBell';

interface AdminDashboardProps {
  onLogout: () => void;
  userData?: any;
}

export default function AdminDashboard({ onLogout, userData }: AdminDashboardProps) {
  const [activeTab, setActiveTab] = useState('overview');

  return (
    <div className="min-h-screen bg-slate-50">
      <div className="bg-white border-b sticky top-0 z-50">
        <div className="max-w-[1800px] mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-600 to-purple-600 flex items-center justify-center">
                <span className="text-white font-bold">G</span>
              </div>
              <div>
                <h1 className="text-xl font-semibold">GASPOLPTN</h1>
                <p className="text-sm text-muted-foreground">Admin Dashboard</p>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <NotificationBell role="admin" />
              <div className="w-px h-8 bg-slate-200" />
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-full bg-gradient-to-br from-indigo-500 to-purple-500 flex items-center justify-center">
                  <span className="text-white text-sm font-bold">A</span>
                </div>
                <div className="hidden sm:block">
                  <div className="text-sm font-medium">{userData?.name || 'Admin Pusat'}</div>
                  <div className="text-xs text-muted-foreground">Super Admin</div>
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
            <TabsTrigger value="users" className="gap-2">
              <UserCog className="w-4 h-4" />
              <span className="hidden sm:inline">Manajemen User</span>
            </TabsTrigger>
            <TabsTrigger value="questions" className="gap-2">
              <FileText className="w-4 h-4" />
              <span className="hidden sm:inline">Review Soal</span>
            </TabsTrigger>
            <TabsTrigger value="events" className="gap-2">
              <Calendar className="w-4 h-4" />
              <span className="hidden sm:inline">Event</span>
            </TabsTrigger>
            <TabsTrigger value="partners" className="gap-2">
              <School className="w-4 h-4" />
              <span className="hidden sm:inline">Mitra</span>
            </TabsTrigger>
            <TabsTrigger value="analytics" className="gap-2">
              <TrendingUp className="w-4 h-4" />
              <span className="hidden sm:inline">Analytics</span>
            </TabsTrigger>
            <TabsTrigger value="finance" className="gap-2">
              <DollarSign className="w-4 h-4" />
              <span className="hidden sm:inline">Keuangan</span>
            </TabsTrigger>
            <TabsTrigger value="gamification" className="gap-2">
              <Sparkles className="w-4 h-4" />
              <span className="hidden sm:inline">Gamifikasi</span>
            </TabsTrigger>
            <TabsTrigger value="packages" className="gap-2">
              <Package className="w-4 h-4" />
              <span className="hidden sm:inline">Paket</span>
            </TabsTrigger>
            <TabsTrigger value="voucher" className="gap-2">
              <Tag className="w-4 h-4" />
              <span className="hidden sm:inline">Voucher</span>
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview"><AdminOverview onNavigate={setActiveTab} /></TabsContent>
          <TabsContent value="users"><AdminUserManagement /></TabsContent>
          <TabsContent value="questions"><QuestionBank /></TabsContent>
          <TabsContent value="events"><EventManagement /></TabsContent>
          <TabsContent value="partners"><PartnerManagement /></TabsContent>
          <TabsContent value="analytics"><AdminAnalytics /></TabsContent>
          <TabsContent value="finance"><AdminFinance /></TabsContent>
          <TabsContent value="gamification"><AdminGamification /></TabsContent>
          <TabsContent value="packages"><AdminPackageManagement /></TabsContent>
          <TabsContent value="voucher"><AdminVoucher /></TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
