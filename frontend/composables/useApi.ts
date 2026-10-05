import { ApiClient } from '~/services/ApiClient'
import { AuthService } from '~/services/AuthService'
import { QuestionService } from '~/services/QuestionService'
import { TryoutService } from '~/services/TryoutService'
import { SchoolService } from '~/services/SchoolService'
import { UserService } from '~/services/UserService'
import { AnalyticsService } from '~/services/AnalyticsService'
import { EventService } from '~/services/EventService'
import { PackageService } from '~/services/PackageService'
import { TaxonomyService } from '~/services/TaxonomyService'
import { QuestionSetService } from '~/services/QuestionSetService'
import { VoucherService } from '~/services/VoucherService'
import { GamificationService } from '~/services/GamificationService'
import { FinanceService } from '~/services/FinanceService'
import { RationalizationService } from '~/services/RationalizationService'
import { ReportService } from '~/services/ReportService'
import { PublicService } from '~/services/PublicService'
import { PaymentService } from '~/services/PaymentService'
import { EntitlementService } from '~/services/EntitlementService'
import { SimulationService } from '~/services/SimulationService'
import { SettingsService } from '~/services/SettingsService'
import { NotificationService } from '~/services/NotificationService'
import { InstitutionService } from '~/services/InstitutionService'
import { AdmissionDeadlineService } from '~/services/AdmissionDeadlineService'

// Builds a fresh set of OOP service instances bound to the current runtime config +
// auth token. Cheap to construct, so we don't bother memoizing across calls — this
// also sidesteps any SSR cross-request state-leak concerns from a module singleton.
export function useApi() {
  const config = useRuntimeConfig()
  const auth = useAuthStore()

  const client = new ApiClient(config.public.apiBase as string, () => auth.token)

  return {
    client,
    authService: new AuthService(client),
    questionService: new QuestionService(client),
    tryoutService: new TryoutService(client),
    schoolService: new SchoolService(client),
    userService: new UserService(client),
    analyticsService: new AnalyticsService(client),
    eventService: new EventService(client),
    packageService: new PackageService(client),
    taxonomyService: new TaxonomyService(client),
    questionSetService: new QuestionSetService(client),
    voucherService: new VoucherService(client),
    gamificationService: new GamificationService(client),
    financeService: new FinanceService(client),
    rationalizationService: new RationalizationService(client),
    reportService: new ReportService(client),
    publicService: new PublicService(client),
    paymentService: new PaymentService(client),
    entitlementService: new EntitlementService(client),
    simulationService: new SimulationService(client),
    settingsService: new SettingsService(client),
    notificationService: new NotificationService(client),
    institutionService: new InstitutionService(client),
    admissionDeadlineService: new AdmissionDeadlineService(client),
  }
}
