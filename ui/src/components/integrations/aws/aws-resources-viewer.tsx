'use client'

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Loading } from "@/components/loading"
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
  Search,
  CheckCircle2,
  XCircle,
  ChevronLeft,
  ChevronRight,
  HardDrive,
  Server,
  Database,
  Lock,
  Globe,
  AlertTriangle,
} from "lucide-react"
import { useAwsS3Buckets, useAwsEc2Instances, useAwsSecurityGroups, useAwsRdsInstances } from '@/hooks/use-api'
import { formatRelativeTime } from '@/types'
import type { AwsS3Bucket, AwsEc2Instance, AwsSecurityGroup, AwsRdsInstance } from '@/types'

interface AwsResourcesViewerProps {
  integrationId: string
}

function S3BucketDetailSheet({ bucket, open, onClose }: { bucket: AwsS3Bucket | null; open: boolean; onClose: () => void }) {
  if (!bucket) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[600px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <HardDrive className="h-5 w-5 text-orange-500" />
            {bucket.bucket_name}
          </SheetTitle>
          <SheetDescription>{bucket.region}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Created</p>
              <p className="text-sm">{formatRelativeTime(bucket.creation_date)}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Region</p>
              <p className="text-sm">{bucket.region}</p>
            </div>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium">Security Settings</h4>
            <div className="grid grid-cols-2 gap-4">
              <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
                <span className="text-sm">Encryption</span>
                {bucket.encryption_enabled ? (
                  <div className="flex items-center gap-2">
                    <CheckCircle2 className="h-4 w-4 text-green-500" />
                    <span className="text-sm">{bucket.encryption_type || 'Enabled'}</span>
                  </div>
                ) : (
                  <div className="flex items-center gap-2">
                    <XCircle className="h-4 w-4 text-red-500" />
                    <span className="text-sm">Disabled</span>
                  </div>
                )}
              </div>
              <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
                <span className="text-sm">Versioning</span>
                {bucket.versioning_enabled ? (
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-yellow-500" />
                )}
              </div>
              <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
                <span className="text-sm">Public Access Block</span>
                {bucket.public_access_blocked ? (
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                ) : (
                  <div className="flex items-center gap-2">
                    <AlertTriangle className="h-4 w-4 text-red-500" />
                    <span className="text-sm text-red-500">Public!</span>
                  </div>
                )}
              </div>
              <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
                <span className="text-sm">Logging</span>
                {bucket.logging_enabled ? (
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-yellow-500" />
                )}
              </div>
            </div>
          </div>

          {Object.keys(bucket.tags).length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Tags</h4>
              <div className="flex flex-wrap gap-2">
                {Object.entries(bucket.tags).map(([key, value]) => (
                  <Badge key={key} variant="outline">
                    {key}: {value}
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

function EC2InstanceDetailSheet({ instance, open, onClose }: { instance: AwsEc2Instance | null; open: boolean; onClose: () => void }) {
  if (!instance) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[600px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Server className="h-5 w-5 text-blue-500" />
            {instance.tags?.Name || instance.instance_id}
          </SheetTitle>
          <SheetDescription className="font-mono">{instance.instance_id}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="flex items-center gap-2">
            <Badge variant={instance.state === 'running' ? 'success' : instance.state === 'stopped' ? 'secondary' : 'warning'}>
              {instance.state}
            </Badge>
            <Badge variant="outline">{instance.instance_type}</Badge>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Region</p>
              <p className="text-sm">{instance.region}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Availability Zone</p>
              <p className="text-sm">{instance.availability_zone}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Public IP</p>
              <p className="text-sm font-mono">{instance.public_ip || 'None'}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Private IP</p>
              <p className="text-sm font-mono">{instance.private_ip || 'None'}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">VPC</p>
              <p className="text-sm font-mono">{instance.vpc_id || 'None'}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Subnet</p>
              <p className="text-sm font-mono">{instance.subnet_id || 'None'}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">IAM Role</p>
              <p className="text-sm">{instance.iam_role || 'None'}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Monitoring</p>
              {instance.monitoring_enabled ? (
                <CheckCircle2 className="h-4 w-4 text-green-500" />
              ) : (
                <XCircle className="h-4 w-4 text-yellow-500" />
              )}
            </div>
          </div>

          <div>
            <p className="text-sm text-muted-foreground">Launch Time</p>
            <p className="text-sm">{formatRelativeTime(instance.launch_time)}</p>
          </div>

          {instance.security_groups.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Security Groups</h4>
              <div className="flex flex-wrap gap-2">
                {instance.security_groups.map((sg) => (
                  <Badge key={sg} variant="outline">
                    <Lock className="h-3 w-3 mr-1" />
                    {sg}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {Object.keys(instance.tags).length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Tags</h4>
              <div className="flex flex-wrap gap-2">
                {Object.entries(instance.tags).map(([key, value]) => (
                  <Badge key={key} variant="secondary">
                    {key}: {value}
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

function RDSInstanceDetailSheet({ instance, open, onClose }: { instance: AwsRdsInstance | null; open: boolean; onClose: () => void }) {
  if (!instance) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[600px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Database className="h-5 w-5 text-purple-500" />
            {instance.db_instance_identifier}
          </SheetTitle>
          <SheetDescription>{instance.engine} {instance.engine_version}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="flex items-center gap-2">
            <Badge variant={instance.status === 'available' ? 'success' : 'warning'}>
              {instance.status}
            </Badge>
            <Badge variant="outline">{instance.db_instance_class}</Badge>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Region</p>
              <p className="text-sm">{instance.region}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Availability Zone</p>
              <p className="text-sm">{instance.availability_zone}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Storage</p>
              <p className="text-sm">{instance.allocated_storage} GB ({instance.storage_type})</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Endpoint</p>
              <p className="text-sm font-mono text-xs">{instance.endpoint || 'N/A'}:{instance.port}</p>
            </div>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium">Security Settings</h4>
            <div className="grid grid-cols-2 gap-4">
              <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
                <span className="text-sm">Multi-AZ</span>
                {instance.multi_az ? (
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-yellow-500" />
                )}
              </div>
              <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
                <span className="text-sm">Encryption</span>
                {instance.storage_encrypted ? (
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-red-500" />
                )}
              </div>
              <div className="flex items-center justify-between p-3 bg-muted rounded-lg col-span-2">
                <span className="text-sm">Publicly Accessible</span>
                {instance.publicly_accessible ? (
                  <div className="flex items-center gap-2">
                    <AlertTriangle className="h-4 w-4 text-red-500" />
                    <span className="text-sm text-red-500">Yes - Security Risk!</span>
                  </div>
                ) : (
                  <div className="flex items-center gap-2">
                    <CheckCircle2 className="h-4 w-4 text-green-500" />
                    <span className="text-sm">No</span>
                  </div>
                )}
              </div>
            </div>
          </div>

          {instance.security_groups.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Security Groups</h4>
              <div className="flex flex-wrap gap-2">
                {instance.security_groups.map((sg) => (
                  <Badge key={sg} variant="outline">
                    <Lock className="h-3 w-3 mr-1" />
                    {sg}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          <div>
            <p className="text-sm text-muted-foreground">Created</p>
            <p className="text-sm">{formatRelativeTime(instance.created_time)}</p>
          </div>
        </div>
      </SheetContent>
    </Sheet>
  )
}

export function AwsResourcesViewer({ integrationId }: AwsResourcesViewerProps) {
  const [activeTab, setActiveTab] = useState('s3')
  const [searchQuery, setSearchQuery] = useState('')
  const [page, setPage] = useState(0)
  const [selectedBucket, setSelectedBucket] = useState<AwsS3Bucket | null>(null)
  const [selectedInstance, setSelectedInstance] = useState<AwsEc2Instance | null>(null)
  const [selectedRds, setSelectedRds] = useState<AwsRdsInstance | null>(null)
  const limit = 25

  const { data: bucketsData, isLoading: bucketsLoading } = useAwsS3Buckets(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
  })

  const { data: instancesData, isLoading: instancesLoading } = useAwsEc2Instances(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
  })

  const { data: rdsData, isLoading: rdsLoading } = useAwsRdsInstances(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
  })

  const isLoading = bucketsLoading || instancesLoading || rdsLoading

  if (isLoading && page === 0) {
    return <Loading message="Loading resources..." />
  }

  const buckets = bucketsData?.data || []
  const instances = instancesData?.data || []
  const rdsInstances = rdsData?.data || []
  const totalBuckets = bucketsData?.total || 0
  const totalInstances = instancesData?.total || 0
  const totalRds = rdsData?.total || 0

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search resources..."
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value)
              setPage(0)
            }}
            className="pl-10"
          />
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={(v) => { setActiveTab(v); setPage(0); }}>
        <TabsList>
          <TabsTrigger value="s3" className="gap-2">
            <HardDrive className="h-4 w-4" />
            S3 Buckets ({totalBuckets})
          </TabsTrigger>
          <TabsTrigger value="ec2" className="gap-2">
            <Server className="h-4 w-4" />
            EC2 Instances ({totalInstances})
          </TabsTrigger>
          <TabsTrigger value="rds" className="gap-2">
            <Database className="h-4 w-4" />
            RDS Instances ({totalRds})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="s3" className="mt-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Bucket Name</TableHead>
                  <TableHead>Region</TableHead>
                  <TableHead>Encryption</TableHead>
                  <TableHead>Versioning</TableHead>
                  <TableHead>Public Access</TableHead>
                  <TableHead>Created</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {buckets.map((bucket) => (
                  <TableRow
                    key={bucket.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedBucket(bucket)}
                  >
                    <TableCell className="font-medium">{bucket.bucket_name}</TableCell>
                    <TableCell>{bucket.region}</TableCell>
                    <TableCell>
                      {bucket.encryption_enabled ? (
                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                      ) : (
                        <XCircle className="h-4 w-4 text-red-500" />
                      )}
                    </TableCell>
                    <TableCell>
                      {bucket.versioning_enabled ? (
                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                      ) : (
                        <XCircle className="h-4 w-4 text-yellow-500" />
                      )}
                    </TableCell>
                    <TableCell>
                      {bucket.public_access_blocked ? (
                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                      ) : (
                        <div className="flex items-center gap-1">
                          <Globe className="h-4 w-4 text-red-500" />
                          <span className="text-xs text-red-500">Public</span>
                        </div>
                      )}
                    </TableCell>
                    <TableCell>{formatRelativeTime(bucket.creation_date)}</TableCell>
                  </TableRow>
                ))}
                {buckets.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                      No S3 buckets found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </Card>
          <Pagination
            page={page}
            setPage={setPage}
            total={totalBuckets}
            limit={limit}
          />
        </TabsContent>

        <TabsContent value="ec2" className="mt-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name / Instance ID</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>State</TableHead>
                  <TableHead>Public IP</TableHead>
                  <TableHead>Private IP</TableHead>
                  <TableHead>Region</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {instances.map((instance) => (
                  <TableRow
                    key={instance.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedInstance(instance)}
                  >
                    <TableCell>
                      <div>
                        <p className="font-medium">{instance.tags?.Name || '-'}</p>
                        <p className="text-xs text-muted-foreground font-mono">{instance.instance_id}</p>
                      </div>
                    </TableCell>
                    <TableCell>{instance.instance_type}</TableCell>
                    <TableCell>
                      <Badge variant={instance.state === 'running' ? 'success' : instance.state === 'stopped' ? 'secondary' : 'warning'}>
                        {instance.state}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-mono text-sm">{instance.public_ip || '-'}</TableCell>
                    <TableCell className="font-mono text-sm">{instance.private_ip || '-'}</TableCell>
                    <TableCell>{instance.region}</TableCell>
                  </TableRow>
                ))}
                {instances.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                      No EC2 instances found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </Card>
          <Pagination
            page={page}
            setPage={setPage}
            total={totalInstances}
            limit={limit}
          />
        </TabsContent>

        <TabsContent value="rds" className="mt-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>DB Identifier</TableHead>
                  <TableHead>Engine</TableHead>
                  <TableHead>Class</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Multi-AZ</TableHead>
                  <TableHead>Encrypted</TableHead>
                  <TableHead>Public</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rdsInstances.map((rds) => (
                  <TableRow
                    key={rds.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedRds(rds)}
                  >
                    <TableCell className="font-medium">{rds.db_instance_identifier}</TableCell>
                    <TableCell>{rds.engine} {rds.engine_version}</TableCell>
                    <TableCell>{rds.db_instance_class}</TableCell>
                    <TableCell>
                      <Badge variant={rds.status === 'available' ? 'success' : 'warning'}>
                        {rds.status}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {rds.multi_az ? (
                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                      ) : (
                        <XCircle className="h-4 w-4 text-yellow-500" />
                      )}
                    </TableCell>
                    <TableCell>
                      {rds.storage_encrypted ? (
                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                      ) : (
                        <XCircle className="h-4 w-4 text-red-500" />
                      )}
                    </TableCell>
                    <TableCell>
                      {rds.publicly_accessible ? (
                        <AlertTriangle className="h-4 w-4 text-red-500" />
                      ) : (
                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                      )}
                    </TableCell>
                  </TableRow>
                ))}
                {rdsInstances.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={7} className="text-center text-muted-foreground py-8">
                      No RDS instances found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </Card>
          <Pagination
            page={page}
            setPage={setPage}
            total={totalRds}
            limit={limit}
          />
        </TabsContent>
      </Tabs>

      <S3BucketDetailSheet
        bucket={selectedBucket}
        open={!!selectedBucket}
        onClose={() => setSelectedBucket(null)}
      />
      <EC2InstanceDetailSheet
        instance={selectedInstance}
        open={!!selectedInstance}
        onClose={() => setSelectedInstance(null)}
      />
      <RDSInstanceDetailSheet
        instance={selectedRds}
        open={!!selectedRds}
        onClose={() => setSelectedRds(null)}
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
