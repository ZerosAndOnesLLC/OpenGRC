"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import {
  LayoutDashboard,
  Shield,
  CheckSquare,
  FileText,
  AlertTriangle,
  Users,
  HardDrive,
  UserCheck,
  FileCheck,
  ListTodo,
  BarChart3,
  Settings,
  Puzzle,
  TrendingUp,
  Target,
  Scale,
  FileBarChart,
} from "lucide-react"
import { cn } from "@/lib/utils"
import { Separator } from "@/components/ui/separator"

const navigation = [
  { name: "Dashboard", href: "/", icon: LayoutDashboard },
  { name: "Frameworks", href: "/frameworks/", icon: Shield },
  { name: "Controls", href: "/controls/", icon: CheckSquare },
  { name: "Evidence", href: "/evidence/", icon: FileText },
  { name: "Policies", href: "/policies/", icon: FileCheck },
  { name: "Risks", href: "/risks/", icon: AlertTriangle },
]

const secondaryNavigation = [
  { name: "Vendors", href: "/vendors/", icon: Users },
  { name: "Assets", href: "/assets/", icon: HardDrive },
  { name: "Access Reviews", href: "/access-reviews/", icon: UserCheck },
]

const analyticsNavigation = [
  { name: "Executive Dashboard", href: "/analytics/", icon: LayoutDashboard },
  { name: "Compliance Trends", href: "/analytics/trends/", icon: TrendingUp },
  { name: "Risk Predictions", href: "/analytics/predictions/", icon: Target },
  { name: "Benchmarks", href: "/analytics/benchmarks/", icon: Scale },
  { name: "Report Builder", href: "/analytics/reports/", icon: FileBarChart },
]

const bottomNavigation = [
  { name: "Audits", href: "/audits/", icon: FileCheck },
  { name: "Tasks", href: "/tasks/", icon: ListTodo },
  { name: "Reports", href: "/reports/", icon: BarChart3 },
]

const settingsNavigation = [
  { name: "Integrations", href: "/integrations/", icon: Puzzle },
  { name: "Settings", href: "/settings/", icon: Settings },
]

export function Sidebar() {
  const pathname = usePathname()

  return (
    <div className="flex h-full w-64 flex-col border-r bg-card">
      {/* Logo */}
      <div className="flex h-16 items-center border-b px-6">
        <Link href="/" className="flex items-center space-x-2">
          <Shield className="h-6 w-6 text-primary" />
          <span className="text-xl font-bold">OpenGRC</span>
        </Link>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 overflow-y-auto px-3 py-4">
        {/* Main Navigation */}
        <div className="space-y-1">
          {navigation.map((item) => {
            const isActive = pathname === item.href
            return (
              <Link
                key={item.name}
                href={item.href}
                className={cn(
                  "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                <item.icon className="h-5 w-5" />
                {item.name}
              </Link>
            )
          })}
        </div>

        <Separator className="my-4" />

        {/* Secondary Navigation */}
        <div className="space-y-1">
          {secondaryNavigation.map((item) => {
            const isActive = pathname === item.href
            return (
              <Link
                key={item.name}
                href={item.href}
                className={cn(
                  "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                <item.icon className="h-5 w-5" />
                {item.name}
              </Link>
            )
          })}
        </div>

        <Separator className="my-4" />

        {/* Analytics Navigation */}
        <div className="space-y-1">
          <span className="px-3 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Analytics</span>
          {analyticsNavigation.map((item) => {
            const isActive = pathname === item.href || (item.href !== '/analytics/' && pathname.startsWith(item.href))
            return (
              <Link
                key={item.name}
                href={item.href}
                className={cn(
                  "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                <item.icon className="h-5 w-5" />
                {item.name}
              </Link>
            )
          })}
        </div>

        <Separator className="my-4" />

        {/* Bottom Navigation */}
        <div className="space-y-1">
          {bottomNavigation.map((item) => {
            const isActive = pathname === item.href
            return (
              <Link
                key={item.name}
                href={item.href}
                className={cn(
                  "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                <item.icon className="h-5 w-5" />
                {item.name}
              </Link>
            )
          })}
        </div>

        <Separator className="my-4" />

        {/* Settings Navigation */}
        <div className="space-y-1">
          {settingsNavigation.map((item) => {
            const isActive = pathname === item.href
            return (
              <Link
                key={item.name}
                href={item.href}
                className={cn(
                  "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                <item.icon className="h-5 w-5" />
                {item.name}
              </Link>
            )
          })}
        </div>
      </nav>
    </div>
  )
}
