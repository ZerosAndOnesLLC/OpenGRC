'use client'

import { use, useState } from 'react'
import { useRouter } from 'next/navigation'
import Link from 'next/link'
import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Loading } from "@/components/loading"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Progress } from "@/components/ui/progress"
import {
  ArrowLeft,
  RefreshCw,
  Shield,
  Users,
  Key,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Cloud,
  Database,
  Server,
  HardDrive,
  Activity,
  AlertCircle,
  Lock,
  Eye,
  FileCheck,
} from "lucide-react"
import { useToast } from "@/hooks/use-toast"
import { useIntegration, useAwsOverview, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import { formatDateTime, formatRelativeTime } from '@/types'
import { AwsIamViewer } from '@/components/integrations/aws/aws-iam-viewer'
import { AwsSecurityViewer } from '@/components/integrations/aws/aws-security-viewer'
import { AwsResourcesViewer } from '@/components/integrations/aws/aws-resources-viewer'
import { AwsCloudTrailViewer } from '@/components/integrations/aws/aws-cloudtrail-viewer'

function OverviewStats({ overview }: { overview: import('@/types').AwsOverview }) {
  const mfaPercentage = overview.iam_stats.total_users > 0
    ? Math.round((overview.iam_stats.users_with_mfa / overview.iam_stats.total_users) * 100)
    : 0

  const configCompliancePercentage = overview.config_stats.total_rules > 0
    ? Math.round((overview.config_stats.compliant / overview.config_stats.total_rules) * 100)
    : 0

  return (
    <div className="space-y-6">
      {/* IAM Overview */}
      <div>
        <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <Users className="h-5 w-5" />
          IAM Overview
        </h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <Card>
            <CardContent className="pt-4">
              <div className="text-2xl font-bold">{overview.iam_stats.total_users}</div>
              <p className="text-sm text-muted-foreground">IAM Users</p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4">
              <div className="text-2xl font-bold">{overview.iam_stats.total_roles}</div>
              <p className="text-sm text-muted-foreground">IAM Roles</p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4">
              <div className="flex items-center gap-2">
                <div className="text-2xl font-bold">{mfaPercentage}%</div>
                {mfaPercentage >= 80 ? (
                  <CheckCircle2 className="h-5 w-5 text-green-500" />
                ) : mfaPercentage >= 50 ? (
                  <AlertTriangle className="h-5 w-5 text-yellow-500" />
                ) : (
                  <XCircle className="h-5 w-5 text-red-500" />
                )}
              </div>
              <p className="text-sm text-muted-foreground">MFA Enabled</p>
              <Progress value={mfaPercentage} className="mt-2 h-1" />
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4">
              <div className="flex items-center gap-2">
                <div className="text-2xl font-bold">{overview.iam_stats.admin_policies}</div>
                {overview.iam_stats.admin_policies > 3 && (
                  <AlertTriangle className="h-5 w-5 text-yellow-500" />
                )}
              </div>
              <p className="text-sm text-muted-foreground">Admin Policies</p>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Security Findings */}
      <div>
        <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <Shield className="h-5 w-5" />
          Security Findings
        </h3>
        <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
          <Card>
            <CardContent className="pt-4">
              <div className="text-2xl font-bold">{overview.security_stats.total_findings}</div>
              <p className="text-sm text-muted-foreground">Total</p>
            </CardContent>
          </Card>
          <Card className="border-red-200 dark:border-red-900">
            <CardContent className="pt-4">
              <div className="text-2xl font-bold text-red-600">{overview.security_stats.critical}</div>
              <p className="text-sm text-muted-foreground">Critical</p>
            </CardContent>
          </Card>
          <Card className="border-orange-200 dark:border-orange-900">
            <CardContent className="pt-4">
              <div className="text-2xl font-bold text-orange-600">{overview.security_stats.high}</div>
              <p className="text-sm text-muted-foreground">High</p>
            </CardContent>
          </Card>
          <Card className="border-yellow-200 dark:border-yellow-900">
            <CardContent className="pt-4">
              <div className="text-2xl font-bold text-yellow-600">{overview.security_stats.medium}</div>
              <p className="text-sm text-muted-foreground">Medium</p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4">
              <div className="text-2xl font-bold text-blue-600">{overview.security_stats.low}</div>
              <p className="text-sm text-muted-foreground">Low</p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4">
              <div className="text-2xl font-bold text-gray-600">{overview.security_stats.informational}</div>
              <p className="text-sm text-muted-foreground">Info</p>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Config Compliance */}
      <div>
        <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <CheckCircle2 className="h-5 w-5" />
          Config Compliance
        </h3>
        <Card>
          <CardContent className="pt-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm text-muted-foreground">
                {overview.config_stats.compliant} of {overview.config_stats.total_rules} rules compliant
              </span>
              <span className="text-lg font-bold">{configCompliancePercentage}%</span>
            </div>
            <Progress value={configCompliancePercentage} className="h-2" />
            <div className="flex gap-4 mt-4">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-green-500" />
                <span className="text-sm">{overview.config_stats.compliant} Compliant</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-red-500" />
                <span className="text-sm">{overview.config_stats.non_compliant} Non-Compliant</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-gray-400" />
                <span className="text-sm">{overview.config_stats.not_applicable} N/A</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Resources */}
      <div>
        <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <Cloud className="h-5 w-5" />
          Resources
        </h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <Card>
            <CardContent className="pt-4 flex items-center gap-3">
              <HardDrive className="h-8 w-8 text-orange-500" />
              <div>
                <div className="text-2xl font-bold">{overview.resource_stats.s3_buckets}</div>
                <p className="text-sm text-muted-foreground">S3 Buckets</p>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4 flex items-center gap-3">
              <Server className="h-8 w-8 text-blue-500" />
              <div>
                <div className="text-2xl font-bold">{overview.resource_stats.ec2_instances}</div>
                <p className="text-sm text-muted-foreground">EC2 Instances</p>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4 flex items-center gap-3">
              <Database className="h-8 w-8 text-purple-500" />
              <div>
                <div className="text-2xl font-bold">{overview.resource_stats.rds_instances}</div>
                <p className="text-sm text-muted-foreground">RDS Instances</p>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4 flex items-center gap-3">
              <Lock className="h-8 w-8 text-green-500" />
              <div>
                <div className="text-2xl font-bold">{overview.resource_stats.security_groups}</div>
                <p className="text-sm text-muted-foreground">Security Groups</p>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* CloudTrail */}
      <div>
        <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <Activity className="h-5 w-5" />
          CloudTrail (24h)
        </h3>
        <div className="grid grid-cols-3 gap-4">
          <Card>
            <CardContent className="pt-4">
              <div className="text-2xl font-bold">{overview.cloudtrail_stats.total_events_24h}</div>
              <p className="text-sm text-muted-foreground">Total Events</p>
            </CardContent>
          </Card>
          <Card className={overview.cloudtrail_stats.root_events > 0 ? "border-red-200 dark:border-red-900" : ""}>
            <CardContent className="pt-4">
              <div className="flex items-center gap-2">
                <div className="text-2xl font-bold">{overview.cloudtrail_stats.root_events}</div>
                {overview.cloudtrail_stats.root_events > 0 && (
                  <AlertCircle className="h-5 w-5 text-red-500" />
                )}
              </div>
              <p className="text-sm text-muted-foreground">Root Activity</p>
            </CardContent>
          </Card>
          <Card className={overview.cloudtrail_stats.sensitive_events > 10 ? "border-yellow-200 dark:border-yellow-900" : ""}>
            <CardContent className="pt-4">
              <div className="flex items-center gap-2">
                <div className="text-2xl font-bold">{overview.cloudtrail_stats.sensitive_events}</div>
                {overview.cloudtrail_stats.sensitive_events > 10 && (
                  <Eye className="h-5 w-5 text-yellow-500" />
                )}
              </div>
              <p className="text-sm text-muted-foreground">Sensitive Actions</p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

export default function AwsDashboardPage({ params }: { params: Promise<{ id: string }> }) {
  const resolvedParams = use(params)
  const integrationId = resolvedParams.id
  const router = useRouter()
  const [activeTab, setActiveTab] = useState('overview')
  const { toast } = useToast()

  const { data: integrationData, isLoading: integrationLoading } = useIntegration(integrationId)
  const { data: overviewData, isLoading: overviewLoading, refetch: refetchOverview } = useAwsOverview(integrationId)

  const syncMutation = useMutation(async (id: string) => {
    return apiClient.post(`/integrations/${id}/sync`, { full_sync: false })
  })

  const collectEvidenceMutation = useMutation(async (id: string) => {
    return apiClient.post<{ data: { evidence_created: number } }>(`/integrations/${id}/collect-evidence`, {})
  })

  const handleSync = async () => {
    try {
      await syncMutation.mutate(integrationId)
      refetchOverview()
    } catch (error) {
      console.error('Sync failed:', error)
    }
  }

  const handleCollectEvidence = async () => {
    try {
      const result = await collectEvidenceMutation.mutate(integrationId)
      toast({
        title: "Evidence Collected",
        description: `Created ${result.data.evidence_created} evidence records from AWS data.`,
      })
      refetchOverview()
    } catch (error) {
      console.error('Evidence collection failed:', error)
      toast({
        title: "Evidence Collection Failed",
        description: error instanceof Error ? error.message : "An error occurred while collecting evidence.",
        variant: "destructive",
      })
    }
  }

  if (integrationLoading || overviewLoading) {
    return <Loading message="Loading AWS dashboard..." />
  }

  const integration = integrationData?.data
  const overview = overviewData?.data

  if (!integration) {
    return (
      <div className="flex flex-col items-center justify-center py-12">
        <AlertTriangle className="h-12 w-12 text-yellow-500 mb-4" />
        <h2 className="text-xl font-semibold mb-2">Integration Not Found</h2>
        <p className="text-muted-foreground mb-4">The requested AWS integration could not be found.</p>
        <Button asChild>
          <Link href="/integrations/">Back to Integrations</Link>
        </Button>
      </div>
    )
  }

  if (integration.integration.integration_type !== 'aws') {
    router.push('/integrations/')
    return null
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={integration.integration.name}
        description={`AWS Account: ${overview?.account_id || 'Loading...'}`}
        action={
          <div className="flex gap-2">
            <Button variant="outline" asChild>
              <Link href="/integrations/">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Back
              </Link>
            </Button>
            <Button
              variant="outline"
              onClick={handleCollectEvidence}
              disabled={collectEvidenceMutation.isLoading || integration.integration.status === 'syncing'}
            >
              <FileCheck className={`mr-2 h-4 w-4 ${collectEvidenceMutation.isLoading ? 'animate-pulse' : ''}`} />
              {collectEvidenceMutation.isLoading ? 'Collecting...' : 'Collect Evidence'}
            </Button>
            <Button
              onClick={handleSync}
              disabled={integration.integration.status === 'syncing' || syncMutation.isLoading}
            >
              <RefreshCw className={`mr-2 h-4 w-4 ${integration.integration.status === 'syncing' || syncMutation.isLoading ? 'animate-spin' : ''}`} />
              Sync Now
            </Button>
          </div>
        }
      />

      <div className="flex items-center gap-4">
        <Badge variant={integration.integration.status === 'active' ? 'success' : integration.integration.status === 'error' ? 'destructive' : 'secondary'}>
          {integration.integration.status}
        </Badge>
        <span className="text-sm text-muted-foreground">
          Last synced: {formatRelativeTime(integration.integration.last_sync_at)}
        </span>
      </div>

      {integration.integration.last_error && (
        <Card className="border-destructive">
          <CardContent className="pt-4">
            <div className="flex items-start gap-3">
              <AlertCircle className="h-5 w-5 text-destructive mt-0.5" />
              <div>
                <p className="font-medium text-destructive">Last Sync Error</p>
                <p className="text-sm text-muted-foreground">{integration.integration.last_error}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="iam">IAM</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="resources">Resources</TabsTrigger>
          <TabsTrigger value="cloudtrail">CloudTrail</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="mt-6">
          {overview ? (
            <OverviewStats overview={overview} />
          ) : (
            <Card>
              <CardContent className="py-12 text-center">
                <Cloud className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-lg font-medium mb-2">No Data Available</h3>
                <p className="text-muted-foreground mb-4">
                  Sync this integration to start collecting AWS data.
                </p>
                <Button onClick={handleSync} disabled={syncMutation.isLoading}>
                  <RefreshCw className={`mr-2 h-4 w-4 ${syncMutation.isLoading ? 'animate-spin' : ''}`} />
                  Start Sync
                </Button>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="iam" className="mt-6">
          <AwsIamViewer integrationId={integrationId} />
        </TabsContent>

        <TabsContent value="security" className="mt-6">
          <AwsSecurityViewer integrationId={integrationId} />
        </TabsContent>

        <TabsContent value="resources" className="mt-6">
          <AwsResourcesViewer integrationId={integrationId} />
        </TabsContent>

        <TabsContent value="cloudtrail" className="mt-6">
          <AwsCloudTrailViewer integrationId={integrationId} />
        </TabsContent>
      </Tabs>
    </div>
  )
}
