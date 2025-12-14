'use client'

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Loading } from "@/components/loading"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import {
  Search,
  ChevronLeft,
  ChevronRight,
  Activity,
  AlertCircle,
  User,
  Eye,
  Clock,
  Globe,
  XCircle,
  Shield,
} from "lucide-react"
import { useAwsCloudTrailEvents, useAwsCloudTrailStats } from '@/hooks/use-api'
import { formatDateTime, formatRelativeTime } from '@/types'
import type { AwsCloudTrailEvent } from '@/types'

interface AwsCloudTrailViewerProps {
  integrationId: string
}

const riskColors: Record<string, string> = {
  high: 'destructive',
  medium: 'warning',
  low: 'secondary',
}

function EventDetailSheet({ event, open, onClose }: { event: AwsCloudTrailEvent | null; open: boolean; onClose: () => void }) {
  if (!event) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[700px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            {event.event_name}
          </SheetTitle>
          <SheetDescription>{event.event_source}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="flex items-center gap-2">
            {event.is_root_action && (
              <Badge variant="destructive">Root Activity</Badge>
            )}
            {event.is_sensitive_action && (
              <Badge variant="warning">Sensitive Action</Badge>
            )}
            <Badge variant={riskColors[event.risk_level] as 'destructive' | 'warning' | 'secondary'}>
              {event.risk_level} risk
            </Badge>
            {event.error_code && (
              <Badge variant="destructive">Error: {event.error_code}</Badge>
            )}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Event ID</p>
              <p className="font-mono text-xs break-all">{event.event_id}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Event Time</p>
              <p className="text-sm">{formatDateTime(event.event_time)}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Event Type</p>
              <p className="text-sm">{event.event_type}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Region</p>
              <p className="text-sm">{event.region}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Source IP</p>
              <p className="font-mono text-sm">{event.source_ip_address || 'N/A'}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">User Agent</p>
              <p className="text-xs text-muted-foreground truncate" title={event.user_agent || ''}>
                {event.user_agent || 'N/A'}
              </p>
            </div>
          </div>

          <div>
            <h4 className="font-medium mb-2">User Identity</h4>
            <div className="p-3 bg-muted rounded-lg space-y-2">
              <div className="flex justify-between">
                <span className="text-sm text-muted-foreground">Type</span>
                <span className="text-sm">{event.user_identity.type}</span>
              </div>
              {event.user_identity.user_name && (
                <div className="flex justify-between">
                  <span className="text-sm text-muted-foreground">User Name</span>
                  <span className="text-sm">{event.user_identity.user_name}</span>
                </div>
              )}
              {event.user_identity.arn && (
                <div className="flex justify-between">
                  <span className="text-sm text-muted-foreground">ARN</span>
                  <span className="text-xs font-mono break-all">{event.user_identity.arn}</span>
                </div>
              )}
              {event.user_identity.access_key_id && (
                <div className="flex justify-between">
                  <span className="text-sm text-muted-foreground">Access Key</span>
                  <span className="font-mono text-sm">{event.user_identity.access_key_id}</span>
                </div>
              )}
            </div>
          </div>

          {event.error_message && (
            <div>
              <h4 className="font-medium mb-2 text-red-500">Error Message</h4>
              <p className="text-sm p-3 bg-red-500/10 rounded-lg text-red-700 dark:text-red-400">
                {event.error_message}
              </p>
            </div>
          )}

          {event.request_parameters && Object.keys(event.request_parameters).length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Request Parameters</h4>
              <pre className="p-3 bg-muted rounded-lg text-xs overflow-x-auto max-h-[200px]">
                {JSON.stringify(event.request_parameters, null, 2)}
              </pre>
            </div>
          )}

          {event.response_elements && Object.keys(event.response_elements).length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Response Elements</h4>
              <pre className="p-3 bg-muted rounded-lg text-xs overflow-x-auto max-h-[200px]">
                {JSON.stringify(event.response_elements, null, 2)}
              </pre>
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  )
}

export function AwsCloudTrailViewer({ integrationId }: AwsCloudTrailViewerProps) {
  const [searchQuery, setSearchQuery] = useState('')
  const [riskFilter, setRiskFilter] = useState<string>('all')
  const [rootFilter, setRootFilter] = useState<string>('all')
  const [page, setPage] = useState(0)
  const [selectedEvent, setSelectedEvent] = useState<AwsCloudTrailEvent | null>(null)
  const limit = 50

  const { data: eventsData, isLoading: eventsLoading } = useAwsCloudTrailEvents(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
    ...(riskFilter !== 'all' && { risk_level: riskFilter }),
    ...(rootFilter === 'root' && { is_root: true }),
    ...(rootFilter === 'non_root' && { is_root: false }),
    ...(rootFilter === 'sensitive' && { is_sensitive: true }),
  })

  const { data: statsData } = useAwsCloudTrailStats(integrationId)

  if (eventsLoading && page === 0) {
    return <Loading message="Loading CloudTrail events..." />
  }

  const events = eventsData?.data || []
  const total = eventsData?.total || 0
  const stats = statsData?.data

  return (
    <div className="space-y-4">
      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-2 md:grid-cols-5 gap-4 mb-6">
          <Card>
            <CardContent className="pt-4">
              <div className="flex items-center gap-3">
                <Activity className="h-8 w-8 text-blue-500" />
                <div>
                  <div className="text-2xl font-bold">{stats.total_events_24h.toLocaleString()}</div>
                  <p className="text-sm text-muted-foreground">Events (24h)</p>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card className={stats.root_activity_count > 0 ? "border-red-200 dark:border-red-900" : ""}>
            <CardContent className="pt-4">
              <div className="flex items-center gap-3">
                <Shield className={`h-8 w-8 ${stats.root_activity_count > 0 ? 'text-red-500' : 'text-green-500'}`} />
                <div>
                  <div className="text-2xl font-bold">{stats.root_activity_count}</div>
                  <p className="text-sm text-muted-foreground">Root Activity</p>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card className={stats.sensitive_actions_count > 20 ? "border-yellow-200 dark:border-yellow-900" : ""}>
            <CardContent className="pt-4">
              <div className="flex items-center gap-3">
                <Eye className={`h-8 w-8 ${stats.sensitive_actions_count > 20 ? 'text-yellow-500' : 'text-muted-foreground'}`} />
                <div>
                  <div className="text-2xl font-bold">{stats.sensitive_actions_count}</div>
                  <p className="text-sm text-muted-foreground">Sensitive Actions</p>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card className={stats.error_count > 10 ? "border-red-200 dark:border-red-900" : ""}>
            <CardContent className="pt-4">
              <div className="flex items-center gap-3">
                <XCircle className={`h-8 w-8 ${stats.error_count > 10 ? 'text-red-500' : 'text-muted-foreground'}`} />
                <div>
                  <div className="text-2xl font-bold">{stats.error_count}</div>
                  <p className="text-sm text-muted-foreground">Errors</p>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4">
              <div className="flex items-center gap-3">
                <User className="h-8 w-8 text-purple-500" />
                <div>
                  <div className="text-2xl font-bold">{stats.top_users?.length || 0}</div>
                  <p className="text-sm text-muted-foreground">Active Users</p>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Top Users and Events */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">Top Users (24h)</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {stats.top_users?.slice(0, 5).map((item, idx) => (
                  <div key={idx} className="flex items-center justify-between">
                    <span className="text-sm truncate flex-1">{item.user}</span>
                    <Badge variant="secondary">{item.count}</Badge>
                  </div>
                ))}
                {(!stats.top_users || stats.top_users.length === 0) && (
                  <p className="text-sm text-muted-foreground">No user activity</p>
                )}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">Top Events (24h)</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {stats.top_events?.slice(0, 5).map((item, idx) => (
                  <div key={idx} className="flex items-center justify-between">
                    <span className="text-sm truncate flex-1 font-mono">{item.event}</span>
                    <Badge variant="secondary">{item.count}</Badge>
                  </div>
                ))}
                {(!stats.top_events || stats.top_events.length === 0) && (
                  <p className="text-sm text-muted-foreground">No events</p>
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search events, users, or IPs..."
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value)
              setPage(0)
            }}
            className="pl-10"
          />
        </div>
        <Select value={riskFilter} onValueChange={(v) => { setRiskFilter(v); setPage(0); }}>
          <SelectTrigger className="w-[130px]">
            <SelectValue placeholder="Risk Level" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Risks</SelectItem>
            <SelectItem value="high">High</SelectItem>
            <SelectItem value="medium">Medium</SelectItem>
            <SelectItem value="low">Low</SelectItem>
          </SelectContent>
        </Select>
        <Select value={rootFilter} onValueChange={(v) => { setRootFilter(v); setPage(0); }}>
          <SelectTrigger className="w-[160px]">
            <SelectValue placeholder="Event Type" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Events</SelectItem>
            <SelectItem value="root">Root Only</SelectItem>
            <SelectItem value="sensitive">Sensitive Only</SelectItem>
            <SelectItem value="non_root">Non-Root Only</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Events Table */}
      <Card>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Event Time</TableHead>
              <TableHead>Event Name</TableHead>
              <TableHead>User</TableHead>
              <TableHead>Source IP</TableHead>
              <TableHead>Risk</TableHead>
              <TableHead>Flags</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {events.map((event) => (
              <TableRow
                key={event.id}
                className="cursor-pointer hover:bg-muted/50"
                onClick={() => setSelectedEvent(event)}
              >
                <TableCell className="whitespace-nowrap">
                  <div className="flex items-center gap-2">
                    <Clock className="h-4 w-4 text-muted-foreground" />
                    {formatRelativeTime(event.event_time)}
                  </div>
                </TableCell>
                <TableCell>
                  <div>
                    <p className="font-medium font-mono text-sm">{event.event_name}</p>
                    <p className="text-xs text-muted-foreground">{event.event_source}</p>
                  </div>
                </TableCell>
                <TableCell>
                  <div className="flex items-center gap-2">
                    <User className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm truncate max-w-[150px]">
                      {event.user_identity.user_name || event.user_identity.type}
                    </span>
                  </div>
                </TableCell>
                <TableCell className="font-mono text-sm">
                  {event.source_ip_address || '-'}
                </TableCell>
                <TableCell>
                  <Badge variant={riskColors[event.risk_level] as 'destructive' | 'warning' | 'secondary'}>
                    {event.risk_level}
                  </Badge>
                </TableCell>
                <TableCell>
                  <div className="flex gap-1">
                    {event.is_root_action && (
                      <Badge variant="destructive" className="text-xs">Root</Badge>
                    )}
                    {event.is_sensitive_action && (
                      <Badge variant="warning" className="text-xs">Sensitive</Badge>
                    )}
                    {event.error_code && (
                      <Badge variant="destructive" className="text-xs">Error</Badge>
                    )}
                  </div>
                </TableCell>
              </TableRow>
            ))}
            {events.length === 0 && (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                  No CloudTrail events found
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </Card>

      <Pagination
        page={page}
        setPage={setPage}
        total={total}
        limit={limit}
      />

      <EventDetailSheet
        event={selectedEvent}
        open={!!selectedEvent}
        onClose={() => setSelectedEvent(null)}
      />
    </div>
  )
}

function Pagination({ page, setPage, total, limit }: { page: number; setPage: (p: number) => void; total: number; limit: number }) {
  const totalPages = Math.ceil(total / limit)
  if (totalPages <= 1) return null

  return (
    <div className="flex items-center justify-between mt-4">
      <p className="text-sm text-muted-foreground">
        Showing {page * limit + 1} to {Math.min((page + 1) * limit, total)} of {total}
      </p>
      <div className="flex gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => setPage(page - 1)}
          disabled={page === 0}
        >
          <ChevronLeft className="h-4 w-4" />
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setPage(page + 1)}
          disabled={page >= totalPages - 1}
        >
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
