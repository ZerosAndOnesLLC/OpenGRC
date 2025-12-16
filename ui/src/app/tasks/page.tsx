'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Loading } from "@/components/loading"
import { TaskDetailSheet } from "@/components/task-detail-sheet"
import {
  Plus,
  Search,
  Filter,
  CheckSquare,
  Clock,
  AlertTriangle,
  User,
  Calendar,
  ListTodo,
} from "lucide-react"
import { useTasks, useTaskStats, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type { Task, CreateTask, TaskStats, TaskType, TaskPriority } from '@/types'
import { formatStatus, formatDate } from '@/types'

const statusVariants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning'> = {
  open: 'secondary',
  in_progress: 'default',
  completed: 'success',
  overdue: 'destructive',
}

const priorityVariants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline' | 'warning'> = {
  low: 'outline',
  medium: 'secondary',
  high: 'warning',
  critical: 'destructive',
}

const typeLabels: Record<string, string> = {
  control_test: 'Control Test',
  evidence_collection: 'Evidence Collection',
  review: 'Review',
  remediation: 'Remediation',
  general: 'General',
}

function StatsCards({ stats }: { stats: TaskStats | null }) {
  if (!stats) return null

  return (
    <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Tasks</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <ListTodo className="h-5 w-5 text-primary" />
            <span className="text-2xl font-bold">{stats.total}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Open</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <CheckSquare className="h-5 w-5 text-blue-500" />
            <span className="text-2xl font-bold">{stats.open}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">In Progress</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Clock className="h-5 w-5 text-yellow-500" />
            <span className="text-2xl font-bold">{stats.in_progress}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Overdue</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-red-500" />
            <span className="text-2xl font-bold">{stats.overdue}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Due This Week</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Calendar className="h-5 w-5 text-orange-500" />
            <span className="text-2xl font-bold">{stats.due_this_week}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function TaskForm({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const [formData, setFormData] = useState<CreateTask>({
    title: '',
    description: '',
    task_type: 'general',
    priority: 'medium',
  })

  const createMutation = useMutation(async (data: CreateTask) => {
    return apiClient.post<Task>('/tasks', data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({
        title: '',
        description: '',
        task_type: 'general',
        priority: 'medium',
      })
      onSuccess()
    } catch {
      // Error is handled by mutation
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Create Task</DialogTitle>
          <DialogDescription>
            Create a new task to track work items and assignments.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="title" className="text-right">
                Title *
              </Label>
              <Input
                id="title"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                className="col-span-3"
                placeholder="e.g., Review access control policy"
                required
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="description" className="text-right">
                Description
              </Label>
              <Textarea
                id="description"
                value={formData.description || ''}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                className="col-span-3"
                rows={3}
                placeholder="Describe the task..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="task_type" className="text-right">
                Type
              </Label>
              <Select
                value={formData.task_type}
                onValueChange={(value: TaskType) => setFormData({ ...formData, task_type: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="general">General</SelectItem>
                  <SelectItem value="control_test">Control Test</SelectItem>
                  <SelectItem value="evidence_collection">Evidence Collection</SelectItem>
                  <SelectItem value="review">Review</SelectItem>
                  <SelectItem value="remediation">Remediation</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="priority" className="text-right">
                Priority
              </Label>
              <Select
                value={formData.priority}
                onValueChange={(value: TaskPriority) => setFormData({ ...formData, priority: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select priority" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="low">Low</SelectItem>
                  <SelectItem value="medium">Medium</SelectItem>
                  <SelectItem value="high">High</SelectItem>
                  <SelectItem value="critical">Critical</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="due_at" className="text-right">
                Due Date
              </Label>
              <Input
                id="due_at"
                type="date"
                value={formData.due_at ? new Date(formData.due_at).toISOString().split('T')[0] : ''}
                onChange={(e) => setFormData({ ...formData, due_at: e.target.value ? `${e.target.value}T23:59:59Z` : undefined })}
                className="col-span-3"
              />
            </div>
          </div>
          {createMutation.error && (
            <div className="text-sm text-red-500 mb-4">
              {createMutation.error.message}
            </div>
          )}
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={createMutation.isLoading}>
              {createMutation.isLoading ? 'Creating...' : 'Create Task'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

interface TaskWithAssignee extends Task {
  assignee_name?: string | null
  assignee_email?: string | null
}

export default function TasksPage() {
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [priorityFilter, setPriorityFilter] = useState<string>('')
  const [typeFilter, setTypeFilter] = useState<string>('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null)
  const [isDetailOpen, setIsDetailOpen] = useState(false)

  const query: Record<string, string | number | boolean> = {}
  if (search) query.search = search
  if (statusFilter) query.status = statusFilter
  if (priorityFilter) query.priority = priorityFilter
  if (typeFilter) query.task_type = typeFilter

  const { data: tasks, isLoading, error, refetch } = useTasks(query)
  const { data: stats, refetch: refetchStats } = useTaskStats()

  const handleSuccess = () => {
    refetch()
    refetchStats()
  }

  const handleRowClick = (taskId: string) => {
    setSelectedTaskId(taskId)
    setIsDetailOpen(true)
  }

  const handleDetailClose = () => {
    setIsDetailOpen(false)
    setSelectedTaskId(null)
  }

  const isOverdue = (task: TaskWithAssignee) => {
    return task.due_at && new Date(task.due_at) < new Date() && task.status !== 'completed'
  }

  if (isLoading) {
    return <Loading />
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-red-500 mb-2">Failed to load tasks</p>
          <p className="text-sm text-muted-foreground">{error.message}</p>
          <Button onClick={() => refetch()} className="mt-4">
            Retry
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Tasks"
        description="Manage compliance tasks and workflows"
      >
        <Button onClick={() => setIsCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Task
        </Button>
      </PageHeader>

      <StatsCards stats={stats} />

      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search tasks..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-10"
          />
        </div>
        <div className="flex gap-2">
          <Select value={statusFilter} onValueChange={setStatusFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Statuses</SelectItem>
              <SelectItem value="open">Open</SelectItem>
              <SelectItem value="in_progress">In Progress</SelectItem>
              <SelectItem value="completed">Completed</SelectItem>
            </SelectContent>
          </Select>
          <Select value={priorityFilter} onValueChange={setPriorityFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Priority" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Priorities</SelectItem>
              <SelectItem value="critical">Critical</SelectItem>
              <SelectItem value="high">High</SelectItem>
              <SelectItem value="medium">Medium</SelectItem>
              <SelectItem value="low">Low</SelectItem>
            </SelectContent>
          </Select>
          <Select value={typeFilter} onValueChange={setTypeFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Types</SelectItem>
              <SelectItem value="general">General</SelectItem>
              <SelectItem value="control_test">Control Test</SelectItem>
              <SelectItem value="evidence_collection">Evidence Collection</SelectItem>
              <SelectItem value="review">Review</SelectItem>
              <SelectItem value="remediation">Remediation</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {tasks && tasks.length > 0 ? (
        <div className="rounded-md border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="p-3 text-left text-sm font-medium">Task</th>
                <th className="p-3 text-left text-sm font-medium">Type</th>
                <th className="p-3 text-left text-sm font-medium">Assignee</th>
                <th className="p-3 text-left text-sm font-medium">Priority</th>
                <th className="p-3 text-left text-sm font-medium">Due Date</th>
                <th className="p-3 text-left text-sm font-medium">Status</th>
              </tr>
            </thead>
            <tbody>
              {(tasks as TaskWithAssignee[]).map((task) => (
                <tr
                  key={task.id}
                  className="border-b hover:bg-muted/25 cursor-pointer"
                  onClick={() => handleRowClick(task.id)}
                >
                  <td className="p-3 text-sm">
                    <div>
                      <div className="font-medium">{task.title}</div>
                      {task.description && (
                        <div className="text-muted-foreground text-xs line-clamp-1">
                          {task.description}
                        </div>
                      )}
                    </div>
                  </td>
                  <td className="p-3 text-sm">
                    {typeLabels[task.task_type] || task.task_type}
                  </td>
                  <td className="p-3 text-sm">
                    {task.assignee_name ? (
                      <div className="flex items-center gap-1">
                        <User className="h-3 w-3" />
                        {task.assignee_name}
                      </div>
                    ) : (
                      <span className="text-muted-foreground">Unassigned</span>
                    )}
                  </td>
                  <td className="p-3 text-sm">
                    <Badge variant={priorityVariants[task.priority] || 'secondary'}>
                      {formatStatus(task.priority)}
                    </Badge>
                  </td>
                  <td className="p-3 text-sm">
                    {task.due_at ? (
                      <span className={`flex items-center gap-1 ${isOverdue(task) ? 'text-red-500' : ''}`}>
                        <Clock className="h-3 w-3" />
                        {formatDate(task.due_at)}
                      </span>
                    ) : (
                      '-'
                    )}
                  </td>
                  <td className="p-3 text-sm">
                    <Badge variant={isOverdue(task) ? 'destructive' : statusVariants[task.status] || 'secondary'}>
                      {isOverdue(task) ? 'Overdue' : formatStatus(task.status)}
                    </Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <ListTodo className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No tasks yet</h3>
            <p className="text-muted-foreground text-sm mb-4">
              Create a task to start tracking compliance work items.
            </p>
            <Button onClick={() => setIsCreateOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Create Your First Task
            </Button>
          </CardContent>
        </Card>
      )}

      <TaskForm
        open={isCreateOpen}
        onOpenChange={setIsCreateOpen}
        onSuccess={handleSuccess}
      />

      <TaskDetailSheet
        taskId={selectedTaskId}
        open={isDetailOpen}
        onOpenChange={(open) => {
          if (!open) handleDetailClose()
        }}
        onUpdate={handleSuccess}
        onDelete={handleSuccess}
      />
    </div>
  )
}
