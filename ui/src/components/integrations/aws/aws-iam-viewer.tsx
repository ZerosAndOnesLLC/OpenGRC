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
  Users,
  UserCog,
  FileKey,
  Search,
  CheckCircle2,
  XCircle,
  Key,
  AlertTriangle,
  ChevronLeft,
  ChevronRight,
} from "lucide-react"
import { useAwsIamUsers, useAwsIamRoles, useAwsIamPolicies } from '@/hooks/use-api'
import { formatDateTime, formatRelativeTime } from '@/types'
import type { AwsIamUser, AwsIamRole, AwsIamPolicy } from '@/types'

interface AwsIamViewerProps {
  integrationId: string
}

function UserDetailSheet({ user, open, onClose }: { user: AwsIamUser | null; open: boolean; onClose: () => void }) {
  if (!user) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[600px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle>{user.user_name}</SheetTitle>
          <SheetDescription className="font-mono text-xs">{user.arn}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">User ID</p>
              <p className="font-mono text-sm">{user.user_id}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Created</p>
              <p className="text-sm">{formatDateTime(user.created_date)}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Password Last Used</p>
              <p className="text-sm">{user.password_last_used ? formatRelativeTime(user.password_last_used) : 'Never'}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">MFA Status</p>
              <div className="flex items-center gap-2">
                {user.mfa_enabled ? (
                  <>
                    <CheckCircle2 className="h-4 w-4 text-green-500" />
                    <span className="text-sm">Enabled</span>
                  </>
                ) : (
                  <>
                    <XCircle className="h-4 w-4 text-red-500" />
                    <span className="text-sm">Disabled</span>
                  </>
                )}
              </div>
            </div>
          </div>

          {user.access_keys.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Access Keys</h4>
              <div className="space-y-2">
                {user.access_keys.map((key) => (
                  <div key={key.access_key_id} className="p-3 bg-muted rounded-lg">
                    <div className="flex items-center justify-between">
                      <code className="text-sm">{key.access_key_id}</code>
                      <Badge variant={key.status === 'Active' ? 'success' : 'secondary'}>
                        {key.status}
                      </Badge>
                    </div>
                    <div className="mt-2 text-sm text-muted-foreground">
                      Created: {formatDateTime(key.created_date)}
                      {key.last_used_date && ` | Last used: ${formatRelativeTime(key.last_used_date)}`}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {user.attached_policies.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Attached Policies ({user.attached_policies.length})</h4>
              <div className="flex flex-wrap gap-2">
                {user.attached_policies.map((policy) => (
                  <Badge key={policy} variant="outline">{policy}</Badge>
                ))}
              </div>
            </div>
          )}

          {user.groups.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Groups ({user.groups.length})</h4>
              <div className="flex flex-wrap gap-2">
                {user.groups.map((group) => (
                  <Badge key={group} variant="secondary">{group}</Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  )
}

function RoleDetailSheet({ role, open, onClose }: { role: AwsIamRole | null; open: boolean; onClose: () => void }) {
  if (!role) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[600px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle>{role.role_name}</SheetTitle>
          <SheetDescription className="font-mono text-xs">{role.arn}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Role ID</p>
              <p className="font-mono text-sm">{role.role_id}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Created</p>
              <p className="text-sm">{formatDateTime(role.created_date)}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Last Used</p>
              <p className="text-sm">{role.last_used_at ? formatRelativeTime(role.last_used_at) : 'Never'}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Max Session</p>
              <p className="text-sm">{Math.floor(role.max_session_duration / 3600)}h</p>
            </div>
          </div>

          <div>
            <h4 className="font-medium mb-2">Trust Policy</h4>
            <pre className="p-3 bg-muted rounded-lg text-xs overflow-x-auto">
              {JSON.stringify(JSON.parse(role.assume_role_policy || '{}'), null, 2)}
            </pre>
          </div>

          {role.attached_policies.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Attached Policies ({role.attached_policies.length})</h4>
              <div className="flex flex-wrap gap-2">
                {role.attached_policies.map((policy) => (
                  <Badge key={policy} variant="outline">{policy}</Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  )
}

function PolicyDetailSheet({ policy, open, onClose }: { policy: AwsIamPolicy | null; open: boolean; onClose: () => void }) {
  if (!policy) return null

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[500px] sm:w-[600px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle>{policy.policy_name}</SheetTitle>
          <SheetDescription className="font-mono text-xs">{policy.arn}</SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Policy ID</p>
              <p className="font-mono text-sm">{policy.policy_id}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Attachments</p>
              <p className="text-sm">{policy.attachment_count}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Type</p>
              <Badge variant={policy.is_aws_managed ? 'secondary' : 'outline'}>
                {policy.is_aws_managed ? 'AWS Managed' : 'Customer Managed'}
              </Badge>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Risk Score</p>
              <Badge variant={
                policy.risk_score >= 8 ? 'destructive' :
                policy.risk_score >= 5 ? 'warning' : 'secondary'
              }>
                {policy.risk_score}/10
              </Badge>
            </div>
          </div>

          <div className="flex gap-2">
            {policy.allows_admin_access && (
              <Badge variant="destructive">Admin Access</Badge>
            )}
            {policy.uses_wildcard_resources && (
              <Badge variant="warning">Wildcard Resources</Badge>
            )}
          </div>

          <div>
            <h4 className="font-medium mb-2">Policy Document</h4>
            <pre className="p-3 bg-muted rounded-lg text-xs overflow-x-auto max-h-[400px]">
              {JSON.stringify(JSON.parse(policy.policy_document || '{}'), null, 2)}
            </pre>
          </div>
        </div>
      </SheetContent>
    </Sheet>
  )
}

export function AwsIamViewer({ integrationId }: AwsIamViewerProps) {
  const [activeTab, setActiveTab] = useState('users')
  const [searchQuery, setSearchQuery] = useState('')
  const [page, setPage] = useState(0)
  const [selectedUser, setSelectedUser] = useState<AwsIamUser | null>(null)
  const [selectedRole, setSelectedRole] = useState<AwsIamRole | null>(null)
  const [selectedPolicy, setSelectedPolicy] = useState<AwsIamPolicy | null>(null)
  const limit = 25

  const { data: usersData, isLoading: usersLoading } = useAwsIamUsers(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
  })

  const { data: rolesData, isLoading: rolesLoading } = useAwsIamRoles(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
  })

  const { data: policiesData, isLoading: policiesLoading } = useAwsIamPolicies(integrationId, {
    limit,
    offset: page * limit,
    ...(searchQuery && { search: searchQuery }),
  })

  const isLoading = usersLoading || rolesLoading || policiesLoading

  if (isLoading && page === 0) {
    return <Loading message="Loading IAM data..." />
  }

  const users = usersData?.data || []
  const roles = rolesData?.data || []
  const policies = policiesData?.data || []
  const totalUsers = usersData?.total || 0
  const totalRoles = rolesData?.total || 0
  const totalPolicies = policiesData?.total || 0

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search users, roles, or policies..."
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
          <TabsTrigger value="users" className="gap-2">
            <Users className="h-4 w-4" />
            Users ({totalUsers})
          </TabsTrigger>
          <TabsTrigger value="roles" className="gap-2">
            <UserCog className="h-4 w-4" />
            Roles ({totalRoles})
          </TabsTrigger>
          <TabsTrigger value="policies" className="gap-2">
            <FileKey className="h-4 w-4" />
            Policies ({totalPolicies})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="users" className="mt-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>User Name</TableHead>
                  <TableHead>MFA</TableHead>
                  <TableHead>Access Keys</TableHead>
                  <TableHead>Groups</TableHead>
                  <TableHead>Created</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {users.map((user) => (
                  <TableRow
                    key={user.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedUser(user)}
                  >
                    <TableCell className="font-medium">{user.user_name}</TableCell>
                    <TableCell>
                      {user.mfa_enabled ? (
                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                      ) : (
                        <XCircle className="h-4 w-4 text-red-500" />
                      )}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-1">
                        <Key className="h-4 w-4 text-muted-foreground" />
                        {user.access_keys.length}
                      </div>
                    </TableCell>
                    <TableCell>{user.groups.length}</TableCell>
                    <TableCell>{formatRelativeTime(user.created_date)}</TableCell>
                  </TableRow>
                ))}
                {users.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                      No users found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </Card>
          <Pagination
            page={page}
            setPage={setPage}
            total={totalUsers}
            limit={limit}
          />
        </TabsContent>

        <TabsContent value="roles" className="mt-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Role Name</TableHead>
                  <TableHead>Policies</TableHead>
                  <TableHead>Last Used</TableHead>
                  <TableHead>Created</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {roles.map((role) => (
                  <TableRow
                    key={role.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedRole(role)}
                  >
                    <TableCell className="font-medium">{role.role_name}</TableCell>
                    <TableCell>{role.attached_policies.length}</TableCell>
                    <TableCell>{role.last_used_at ? formatRelativeTime(role.last_used_at) : 'Never'}</TableCell>
                    <TableCell>{formatRelativeTime(role.created_date)}</TableCell>
                  </TableRow>
                ))}
                {roles.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center text-muted-foreground py-8">
                      No roles found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </Card>
          <Pagination
            page={page}
            setPage={setPage}
            total={totalRoles}
            limit={limit}
          />
        </TabsContent>

        <TabsContent value="policies" className="mt-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Policy Name</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Attachments</TableHead>
                  <TableHead>Risk</TableHead>
                  <TableHead>Flags</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {policies.map((policy) => (
                  <TableRow
                    key={policy.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedPolicy(policy)}
                  >
                    <TableCell className="font-medium">{policy.policy_name}</TableCell>
                    <TableCell>
                      <Badge variant={policy.is_aws_managed ? 'secondary' : 'outline'}>
                        {policy.is_aws_managed ? 'AWS' : 'Custom'}
                      </Badge>
                    </TableCell>
                    <TableCell>{policy.attachment_count}</TableCell>
                    <TableCell>
                      <Badge variant={
                        policy.risk_score >= 8 ? 'destructive' :
                        policy.risk_score >= 5 ? 'warning' : 'secondary'
                      }>
                        {policy.risk_score}/10
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <div className="flex gap-1">
                        {policy.allows_admin_access && (
                          <span title="Admin Access">
                            <AlertTriangle className="h-4 w-4 text-red-500" />
                          </span>
                        )}
                        {policy.uses_wildcard_resources && (
                          <span title="Wildcard Resources">
                            <AlertTriangle className="h-4 w-4 text-yellow-500" />
                          </span>
                        )}
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
                {policies.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                      No policies found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </Card>
          <Pagination
            page={page}
            setPage={setPage}
            total={totalPolicies}
            limit={limit}
          />
        </TabsContent>
      </Tabs>

      <UserDetailSheet
        user={selectedUser}
        open={!!selectedUser}
        onClose={() => setSelectedUser(null)}
      />
      <RoleDetailSheet
        role={selectedRole}
        open={!!selectedRole}
        onClose={() => setSelectedRole(null)}
      />
      <PolicyDetailSheet
        policy={selectedPolicy}
        open={!!selectedPolicy}
        onClose={() => setSelectedPolicy(null)}
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
