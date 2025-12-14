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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import {
  Shield,
  AlertTriangle,
  Search,
  CheckCircle2,
  XCircle,
  ChevronLeft,
  ChevronRight,
  Settings,
} from "lucide-react"
import { useAwsFindings, useAwsFindingsSummary, useAwsConfigRules } from '@/hooks/use-api'
import { formatRelativeTime } from '@/types'
import type { AwsSecurityFinding, AwsConfigRule } from '@/types'

interface AwsSecurityViewerProps {
  integrationId: string
}

const severityColors: Record<string, string> = {
  CRITICAL: 'bg-red-600 text-white',
  HIGH: 'bg-orange-500 text-white',
  MEDIUM: 'bg-yellow-500 text-black',
  LOW: 'bg-blue-500 text-white',
  INFORMATIONAL: 'bg-gray-500 text-white',
}

const complianceColors: Record<string, string> = {
  COMPLIANT: 'success',
  NON_COMPLIANT: 'destructive',
  NOT_APPLICABLE: 'secondary',
}

function FindingDetailSheet({ finding, open, onClose }: { finding: AwsSecurityFinding | null; open: boolean; onClose: () => void }) {
  if (!finding) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[700px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Badge className={severityColors[finding.severity_label]}>
              {finding.severity_label}
            </Badge>
            {finding.title}
          </SheetTitle>
          <SheetDescription>{finding.product_name}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div>
            <h4 className="font-medium mb-2">Description</h4>
            <p className="text-sm text-muted-foreground">{finding.description}</p>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Finding ID</p>
              <p className="font-mono text-xs break-all">{finding.finding_id}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Region</p>
              <p className="text-sm">{finding.region}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Workflow Status</p>
              <Badge variant="outline">{finding.workflow_status}</Badge>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Record State</p>
              <Badge variant="outline">{finding.record_state}</Badge>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">First Observed</p>
              <p className="text-sm">{formatRelativeTime(finding.first_observed_at)}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Last Observed</p>
              <p className="text-sm">{formatRelativeTime(finding.last_observed_at)}</p>
            </div>
          </div>

          {finding.types.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Finding Types</h4>
              <div className="flex flex-wrap gap-2">
                {finding.types.map((type, idx) => (
                  <Badge key={idx} variant="outline" className="text-xs">
                    {type}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {finding.compliance_standards.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Compliance Standards</h4>
              <div className="flex flex-wrap gap-2">
                {finding.compliance_standards.map((std, idx) => (
                  <Badge key={idx} variant="secondary">
                    {std}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {finding.related_resources.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Related Resources</h4>
              <div className="space-y-2">
                {finding.related_resources.map((resource, idx) => (
                  <div key={idx} className="p-2 bg-muted rounded text-sm">
                    <p className="font-medium">{resource.type}</p>
                    <p className="font-mono text-xs text-muted-foreground">{resource.id}</p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {finding.remediation && (
            <div>
              <h4 className="font-medium mb-2">Remediation</h4>
              <p className="text-sm text-muted-foreground">{finding.remediation}</p>
            </div>
          )}

          {finding.mapped_control_codes.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Mapped Controls</h4>
              <div className="flex flex-wrap gap-2">
                {finding.mapped_control_codes.map((code) => (
                  <Badge key={code} variant="outline">
                    {code}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  )
}

function ConfigRuleDetailSheet({ rule, open, onClose }: { rule: AwsConfigRule | null; open: boolean; onClose: () => void }) {
  if (!rule) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[600px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle>{rule.config_rule_name}</SheetTitle>
          <SheetDescription className="font-mono text-xs">{rule.config_rule_arn}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="flex items-center gap-2">
            <Badge variant={complianceColors[rule.compliance_type] as 'success' | 'destructive' | 'secondary'}>
              {rule.compliance_type.replace('_', ' ')}
            </Badge>
            <Badge variant="outline">{rule.region}</Badge>
          </div>

          {rule.description && (
            <div>
              <h4 className="font-medium mb-2">Description</h4>
              <p className="text-sm text-muted-foreground">{rule.description}</p>
            </div>
          )}

          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Source</p>
              <p className="text-sm">{rule.source_owner}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Identifier</p>
              <p className="font-mono text-xs">{rule.source_identifier}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Compliant Resources</p>
              <p className="text-lg font-bold text-green-600">{rule.compliant_count}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Non-Compliant Resources</p>
              <p className="text-lg font-bold text-red-600">{rule.non_compliant_count}</p>
            </div>
          </div>

          {rule.mapped_control_codes.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Mapped Controls</h4>
              <div className="flex flex-wrap gap-2">
                {rule.mapped_control_codes.map((code) => (
                  <Badge key={code} variant="outline">
                    {code}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  )
}

export function AwsSecurityViewer({ integrationId }: AwsSecurityViewerProps) {
  const [activeTab, setActiveTab] = useState('findings')
  const [searchQuery, setSearchQuery] = useState('')
  const [severityFilter, setSeverityFilter] = useState<string>('all')
  const [complianceFilter, setComplianceFilter] = useState<string>('all')
  const [page, setPage] = useState(0)
  const [selectedFinding, setSelectedFinding] = useState<AwsSecurityFinding | null>(null)
  const [selectedRule, setSelectedRule] = useState<AwsConfigRule | null>(null)
  const limit = 25

  const { data: findingsData, isLoading: findingsLoading } = useAwsFindings(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
    ...(severityFilter !== 'all' && { severity: severityFilter }),
  })

  const { data: summaryData } = useAwsFindingsSummary(integrationId)

  const { data: rulesData, isLoading: rulesLoading } = useAwsConfigRules(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
    ...(complianceFilter !== 'all' && { compliance_type: complianceFilter }),
  })

  const isLoading = findingsLoading || rulesLoading

  if (isLoading && page === 0) {
    return <Loading message="Loading security data..." />
  }

  const findings = findingsData?.data || []
  const rules = rulesData?.data || []
  const summary = summaryData?.data
  const totalFindings = findingsData?.total || 0
  const totalRules = rulesData?.total || 0

  return (
    <div className="space-y-4">
      {/* Summary Cards */}
      {summary && activeTab === 'findings' && (
        <div className="grid grid-cols-2 md:grid-cols-6 gap-4 mb-6">
          <Card className="border-red-200 dark:border-red-900">
            <CardContent className="pt-4 text-center">
              <div className="text-2xl font-bold text-red-600">{summary.by_severity.critical}</div>
              <p className="text-sm text-muted-foreground">Critical</p>
            </CardContent>
          </Card>
          <Card className="border-orange-200 dark:border-orange-900">
            <CardContent className="pt-4 text-center">
              <div className="text-2xl font-bold text-orange-500">{summary.by_severity.high}</div>
              <p className="text-sm text-muted-foreground">High</p>
            </CardContent>
          </Card>
          <Card className="border-yellow-200 dark:border-yellow-900">
            <CardContent className="pt-4 text-center">
              <div className="text-2xl font-bold text-yellow-600">{summary.by_severity.medium}</div>
              <p className="text-sm text-muted-foreground">Medium</p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4 text-center">
              <div className="text-2xl font-bold text-blue-500">{summary.by_severity.low}</div>
              <p className="text-sm text-muted-foreground">Low</p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4 text-center">
              <div className="text-2xl font-bold text-gray-500">{summary.by_severity.informational}</div>
              <p className="text-sm text-muted-foreground">Info</p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4 text-center">
              <div className="text-2xl font-bold">{summary.total}</div>
              <p className="text-sm text-muted-foreground">Total</p>
            </CardContent>
          </Card>
        </div>
      )}

      <div className="flex items-center gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search findings or rules..."
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value)
              setPage(0)
            }}
            className="pl-10"
          />
        </div>
        {activeTab === 'findings' && (
          <Select value={severityFilter} onValueChange={(v) => { setSeverityFilter(v); setPage(0); }}>
            <SelectTrigger className="w-[150px]">
              <SelectValue placeholder="Severity" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Severities</SelectItem>
              <SelectItem value="CRITICAL">Critical</SelectItem>
              <SelectItem value="HIGH">High</SelectItem>
              <SelectItem value="MEDIUM">Medium</SelectItem>
              <SelectItem value="LOW">Low</SelectItem>
              <SelectItem value="INFORMATIONAL">Info</SelectItem>
            </SelectContent>
          </Select>
        )}
        {activeTab === 'config' && (
          <Select value={complianceFilter} onValueChange={(v) => { setComplianceFilter(v); setPage(0); }}>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="Compliance" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Status</SelectItem>
              <SelectItem value="COMPLIANT">Compliant</SelectItem>
              <SelectItem value="NON_COMPLIANT">Non-Compliant</SelectItem>
              <SelectItem value="NOT_APPLICABLE">Not Applicable</SelectItem>
            </SelectContent>
          </Select>
        )}
      </div>

      <Tabs value={activeTab} onValueChange={(v) => { setActiveTab(v); setPage(0); }}>
        <TabsList>
          <TabsTrigger value="findings" className="gap-2">
            <Shield className="h-4 w-4" />
            Security Hub ({totalFindings})
          </TabsTrigger>
          <TabsTrigger value="config" className="gap-2">
            <Settings className="h-4 w-4" />
            Config Rules ({totalRules})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="findings" className="mt-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Severity</TableHead>
                  <TableHead>Title</TableHead>
                  <TableHead>Product</TableHead>
                  <TableHead>Region</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Last Seen</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {findings.map((finding) => (
                  <TableRow
                    key={finding.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedFinding(finding)}
                  >
                    <TableCell>
                      <Badge className={severityColors[finding.severity_label]}>
                        {finding.severity_label}
                      </Badge>
                    </TableCell>
                    <TableCell className="max-w-[300px] truncate">{finding.title}</TableCell>
                    <TableCell>{finding.product_name}</TableCell>
                    <TableCell>{finding.region}</TableCell>
                    <TableCell>
                      <Badge variant="outline">{finding.workflow_status}</Badge>
                    </TableCell>
                    <TableCell>{formatRelativeTime(finding.last_observed_at)}</TableCell>
                  </TableRow>
                ))}
                {findings.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                      No findings found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </Card>
          <Pagination
            page={page}
            setPage={setPage}
            total={totalFindings}
            limit={limit}
          />
        </TabsContent>

        <TabsContent value="config" className="mt-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Rule Name</TableHead>
                  <TableHead>Compliance</TableHead>
                  <TableHead>Compliant</TableHead>
                  <TableHead>Non-Compliant</TableHead>
                  <TableHead>Region</TableHead>
                  <TableHead>Source</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rules.map((rule) => (
                  <TableRow
                    key={rule.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedRule(rule)}
                  >
                    <TableCell className="font-medium">{rule.config_rule_name}</TableCell>
                    <TableCell>
                      {rule.compliance_type === 'COMPLIANT' ? (
                        <CheckCircle2 className="h-5 w-5 text-green-500" />
                      ) : rule.compliance_type === 'NON_COMPLIANT' ? (
                        <XCircle className="h-5 w-5 text-red-500" />
                      ) : (
                        <Badge variant="secondary">N/A</Badge>
                      )}
                    </TableCell>
                    <TableCell className="text-green-600">{rule.compliant_count}</TableCell>
                    <TableCell className="text-red-600">{rule.non_compliant_count}</TableCell>
                    <TableCell>{rule.region}</TableCell>
                    <TableCell>
                      <Badge variant="outline">{rule.source_owner}</Badge>
                    </TableCell>
                  </TableRow>
                ))}
                {rules.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                      No config rules found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </Card>
          <Pagination
            page={page}
            setPage={setPage}
            total={totalRules}
            limit={limit}
          />
        </TabsContent>
      </Tabs>

      <FindingDetailSheet
        finding={selectedFinding}
        open={!!selectedFinding}
        onClose={() => setSelectedFinding(null)}
      />
      <ConfigRuleDetailSheet
        rule={selectedRule}
        open={!!selectedRule}
        onClose={() => setSelectedRule(null)}
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
