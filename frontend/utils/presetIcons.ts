// Curated preset icon catalogue for the Badge/Package "Pilih Ikon" tab (see
// components/shared/IconPicker.vue) — deliberately a small, hand-picked set of
// achievement/gamification + package/product themed icons, not the entire
// lucide-vue-next library, to keep the picker's UX focused and avoid bundling hundreds of
// unused icon components. Both the picker (choosing an icon_name) and every badge/package
// display location (rendering that icon_name, via IconDisplay.vue) import from this one
// file, so they can never drift out of sync — and the names here match the CASE mapping
// in backend/migrations/20250101000022_badge_package_icons.sql.
import {
  Trophy, Medal, Star, Flame, Zap, Target, Crown, Shield, Gem, Rocket, BookOpen, Brain, Award,
  Sparkles, Heart, CheckCircle2, GraduationCap, Book, Package, Gift, Dumbbell, Sun, ThumbsUp,
  Users, Clock, TrendingUp,
} from 'lucide-vue-next'
import type { Component } from 'vue'

export const PRESET_ICONS: Record<string, Component> = {
  trophy: Trophy,
  medal: Medal,
  star: Star,
  flame: Flame,
  zap: Zap,
  target: Target,
  crown: Crown,
  shield: Shield,
  gem: Gem,
  rocket: Rocket,
  'book-open': BookOpen,
  brain: Brain,
  award: Award,
  sparkles: Sparkles,
  heart: Heart,
  'check-circle': CheckCircle2,
  'graduation-cap': GraduationCap,
  book: Book,
  package: Package,
  gift: Gift,
  dumbbell: Dumbbell,
  sun: Sun,
  'thumbs-up': ThumbsUp,
  users: Users,
  clock: Clock,
  'trending-up': TrendingUp,
}

export const PRESET_ICON_NAMES = Object.keys(PRESET_ICONS)

/** Resolves a preset icon name to its component, falling back to Award for any
 * unknown/legacy name so a badge/package with an unmapped icon never renders blank. */
export function resolveIconComponent(name: string | null | undefined): Component {
  return (name && PRESET_ICONS[name]) || Award
}
