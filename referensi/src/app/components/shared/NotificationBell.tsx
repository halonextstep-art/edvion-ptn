import { useState, useRef, useEffect } from 'react';
import { Bell, CheckCheck, Info, CheckCircle, AlertTriangle, FileSearch, X } from 'lucide-react';
import { useApp, NotifRole, NotifType, Notification } from '../../lib/AppContext';

const typeConfig: Record<NotifType, { icon: typeof Info; color: string; bg: string }> = {
  info:    { icon: Info,        color: 'text-blue-500',   bg: 'bg-blue-50 border-blue-100' },
  success: { icon: CheckCircle, color: 'text-emerald-500',bg: 'bg-emerald-50 border-emerald-100' },
  warning: { icon: AlertTriangle,color: 'text-amber-500', bg: 'bg-amber-50 border-amber-100' },
  review:  { icon: FileSearch,  color: 'text-violet-500', bg: 'bg-violet-50 border-violet-100' },
};

interface Props {
  role: NotifRole;
  dark?: boolean; // for content (dark theme) dashboard
}

export default function NotificationBell({ role, dark = false }: Props) {
  const { notifications, markRead, markAllRead, unreadCount } = useApp();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const count = unreadCount(role);
  const myNotifs = notifications.filter(n => n.forRole === role || n.forRole === 'all');

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  const btnClass = dark
    ? 'relative p-2 rounded-xl hover:bg-white/10 text-white/60 hover:text-white transition-colors'
    : 'relative p-2 rounded-xl hover:bg-slate-100 text-slate-500 hover:text-slate-900 transition-colors';

  return (
    <div className="relative" ref={ref}>
      <button onClick={() => setOpen(o => !o)} className={btnClass}>
        <Bell className="w-5 h-5" />
        {count > 0 && (
          <span className="absolute top-1 right-1 min-w-[16px] h-4 px-1 bg-red-500 text-white text-[9px] font-black rounded-full flex items-center justify-center leading-none">
            {count > 9 ? '9+' : count}
          </span>
        )}
      </button>

      {open && (
        <div className={`absolute right-0 top-full mt-2 w-80 rounded-2xl shadow-2xl border z-[100] overflow-hidden ${dark ? 'bg-[#0f0d1a] border-white/15' : 'bg-white border-slate-200'}`}>
          {/* Header */}
          <div className={`flex items-center justify-between px-4 py-3 border-b ${dark ? 'border-white/10' : 'border-slate-100'}`}>
            <div>
              <p className={`text-sm font-bold ${dark ? 'text-white' : 'text-slate-900'}`}>Notifikasi</p>
              <p className={`text-[10px] ${dark ? 'text-white/40' : 'text-slate-400'}`}>{count} belum dibaca</p>
            </div>
            <div className="flex items-center gap-1">
              {count > 0 && (
                <button onClick={() => markAllRead(role)} className={`flex items-center gap-1 text-[10px] font-semibold px-2 py-1 rounded-lg transition-colors ${dark ? 'text-violet-400 hover:bg-white/10' : 'text-blue-600 hover:bg-blue-50'}`}>
                  <CheckCheck className="w-3 h-3" /> Baca semua
                </button>
              )}
              <button onClick={() => setOpen(false)} className={`p-1.5 rounded-lg transition-colors ${dark ? 'text-white/30 hover:bg-white/10' : 'text-slate-400 hover:bg-slate-100'}`}>
                <X className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          {/* List */}
          <div className="max-h-80 overflow-y-auto">
            {myNotifs.length === 0 && (
              <div className={`py-10 text-center text-sm ${dark ? 'text-white/30' : 'text-slate-400'}`}>
                <Bell className="w-8 h-8 mx-auto mb-2 opacity-20" />
                Tidak ada notifikasi
              </div>
            )}
            {myNotifs.map(n => {
              const cfg = typeConfig[n.type];
              const Icon = cfg.icon;
              return (
                <button
                  key={n.id}
                  onClick={() => markRead(n.id)}
                  className={`w-full text-left flex items-start gap-3 px-4 py-3 border-b transition-colors ${
                    dark
                      ? `border-white/5 hover:bg-white/5 ${!n.read ? 'bg-white/[0.04]' : ''}`
                      : `border-slate-50 hover:bg-slate-50 ${!n.read ? 'bg-blue-50/50' : ''}`
                  }`}
                >
                  <div className={`w-8 h-8 rounded-xl flex items-center justify-center shrink-0 mt-0.5 ${dark ? 'bg-white/10' : cfg.bg} border`}>
                    <Icon className={`w-4 h-4 ${dark ? 'text-violet-400' : cfg.color}`} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between gap-2">
                      <p className={`text-xs font-semibold leading-snug ${dark ? 'text-white/85' : 'text-slate-800'} ${!n.read ? 'font-bold' : ''}`}>{n.title}</p>
                      {!n.read && <span className="w-2 h-2 rounded-full bg-blue-500 shrink-0 mt-1" />}
                    </div>
                    <p className={`text-[11px] leading-snug mt-0.5 ${dark ? 'text-white/45' : 'text-slate-500'}`}>{n.message}</p>
                    <p className={`text-[10px] mt-1 ${dark ? 'text-white/25' : 'text-slate-400'}`}>{n.createdAt}</p>
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
