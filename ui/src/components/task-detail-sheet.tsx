'use client'

import { useState, useEffect } from 'react'
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
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import { Separator } from "@/components/ui/separator"
import { Loading } from "@/components/loading"
import {
  Save,
  Trash2,
  Edit2,
  CheckCircle,
  Clock,
  User,
  Calendar,
  MessageSquare,
  Send,
  ListTodo,
} from "lucide-react"
import { useTask, useTaskComments, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type { Task, UpdateTask, TaskComment, CreateTaskComment } from '@/types'
import { formatStatus, formatDate, formatDateTime } from '@/types'

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

// Extended task with assignee info from API
interface TaskWithAssigneeInfo extends Task {
  assignee_name?: string | null
  assignee_email?: string | null
  created_by_name?: string | null
}

// Comment with user info from API - TaskComment already has user_name/user_email
type CommentWithUserInfo = TaskComment

interface TaskDetailSheetProps {
  taskId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdate?: () => void
  onDelete?: () => void
}

export function TaskDetailSheet({
  taskId,
  open,
  onOpenChange,
  onUpdate,
  onDelete,
}: TaskDetailSheetProps) {
  const { data: taskData, isLoading, refetch } = useTask(taskId || '')
  const { data: commentsData, refetch: refetchComments } = useTaskComments(taskId || '')
  const [isEditing, setIsEditing] = useState(false)
  const [formData, setFormData] = useState<UpdateTask>({})
  const [newComment, setNewComment] = useState('')

  // API returns TaskWithAssignee with flattened task fields
  const task = taskData as TaskWithAssigneeInfo | undefined
  const comments = commentsData as CommentWithUserInfo[] | undefined

  useEffect(() => {
    if (task) {
      setFormData({
        title: task.title,
        description: task.description || '',
        task_type: task.task_type,
        priority: task.priority,
        status: task.status,
        due_at: task.due_at || undefined,
      })
    }
  }, [task])

  useEffect(() => {
    if (!open) {
      setIsEditing(false)
      setNewComment('')
    }
  }, [open])

  const updateMutation = useMutation(async (data: UpdateTask) => {
    return apiClient.put<Task>(`/tasks/${taskId}`, data)
  })

  const deleteMutation = useMutation(async () => {
    return apiClient.delete(`/tasks/${taskId}`)
  })

  const completeMutation = useMutation(async () => {
    return apiClient.post<Task>(`/tasks/${taskId}/complete`, {})
  })

  const addCommentMutation = useMutation(async (data: CreateTaskComment) => {
    return apiClient.post<TaskComment>(`/tasks/${taskId}/comments`, data)
  })

  const handleSave = async () => {
    try {
      await updateMutation.mutate(formData)
      setIsEditing(false)
      refetch()
      onUpdate?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleDelete = async () => {
    if (!confirm('Are you sure you want to delete this task? This action cannot be undone.')) return
    try {
      await deleteMutation.mutate(undefined)
      onOpenChange(false)
      onDelete?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleComplete = async () => {
    try {
      await completeMutation.mutate(undefined)
      refetch()
      onUpdate?.()
    } catch {
      // Error handled
    }
  }

  const handleAddComment = async () => {
    if (!newComment.trim()) return
    try {
      await addCommentMutation.mutate({ content: newComment.trim() })
      setNewComment('')
      refetchComments()
    } catch {
      // Error handled
    }
  }

  const isOverdue = task?.due_at && new Date(task.due_at) < new Date() && task.status !== 'completed'

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="sm:max-w-2xl overflow-y-auto">
        {isLoading || !task ? (
          <div className="flex items-center justify-center h-full">
            <Loading />
          </div>
        ) : (
          <div className="space-y-6">
            <SheetHeader>
              <div className="flex items-center gap-2">
                <ListTodo className="h-5 w-5 text-primary" />
                <SheetTitle className="flex-1">{task.title}</SheetTitle>
                <Badge variant={isOverdue ? 'destructive' : statusVariants[task.status] || 'secondary'}>
                  {isOverdue ? 'Overdue' : formatStatus(task.status)}
                </Badge>
                <Badge variant={priorityVariants[task.priority] || 'secondary'}>
                  {formatStatus(task.priority)}
                </Badge>
              </div>
              <SheetDescription>
                {typeLabels[task.task_type] || task.task_type}
                {task.assignee_name && ` - Assigned to ${task.assignee_name}`}
              </SheetDescription>
            </SheetHeader>

            {/* Actions */}
            <div className="flex gap-2">
              {isEditing ? (
                <>
                  <Button variant="outline" size="sm" onClick={() => setIsEditing(false)}>Cancel</Button>
                  <Button size="sm" onClick={handleSave} disabled={updateMutation.isLoading}>
                    <Save className="mr-2 h-4 w-4" />
                    {updateMutation.isLoading ? 'Saving...' : 'Save'}
                  </Button>
                </>
              ) : (
                <>
                  {task.status !== 'completed' && (
                    <Button
                      size="sm"
                      onClick={handleComplete}
                      disabled={completeMutation.isLoading}
                    >
                      <CheckCircle className="mr-2 h-4 w-4" />
                      {completeMutation.isLoading ? 'Completing...' : 'Complete'}
                    </Button>
                  )}
                  <Button variant="outline" size="sm" onClick={() => setIsEditing(true)}>
                    <Edit2 className="mr-2 h-4 w-4" />
                    Edit
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleDelete}
                    disabled={deleteMutation.isLoading}
                    className="text-destructive hover:text-destructive"
                  >
                    <Trash2 className="mr-2 h-4 w-4" />
                    Delete
                  </Button>
                </>
              )}
            </div>

            <Separator />

            {/* Task Details */}
            <div className="space-y-4">
              <h3 className="font-semibold">Task Details</h3>

              {isEditing ? (
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>Title</Label>
                    <Input
                      value={formData.title || ''}
                      onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Description</Label>
                    <Textarea
                      value={formData.description || ''}
                      onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                      rows={3}
                    />
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label>Type</Label>
                      <Select
                        value={formData.task_type || ''}
                        onValueChange={(value) => setFormData({ ...formData, task_type: value })}
                      >
                        <SelectTrigger><SelectValue /></SelectTrigger>
                        <SelectContent>
                          <SelectItem value="general">General</SelectItem>
                          <SelectItem value="control_test">Control Test</SelectItem>
                          <SelectItem value="evidence_collection">Evidence Collection</SelectItem>
                          <SelectItem value="review">Review</SelectItem>
                          <SelectItem value="remediation">Remediation</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>Priority</Label>
                      <Select
                        value={formData.priority || ''}
                        onValueChange={(value) => setFormData({ ...formData, priority: value })}
                      >
                        <SelectTrigger><SelectValue /></SelectTrigger>
                        <SelectContent>
                          <SelectItem value="low">Low</SelectItem>
                          <SelectItem value="medium">Medium</SelectItem>
                          <SelectItem value="high">High</SelectItem>
                          <SelectItem value="critical">Critical</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label>Status</Label>
                      <Select
                        value={formData.status || ''}
                        onValueChange={(value) => setFormData({ ...formData, status: value })}
                      >
                        <SelectTrigger><SelectValue /></SelectTrigger>
                        <SelectContent>
                          <SelectItem value="open">Open</SelectItem>
                          <SelectItem value="in_progress">In Progress</SelectItem>
                          <SelectItem value="completed">Completed</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>Due Date</Label>
                      <Input
                        type="date"
                        value={formData.due_at ? (typeof formData.due_at === 'string' ? formData.due_at.split('T')[0] : '') : ''}
                        onChange={(e) => setFormData({ ...formData, due_at: e.target.value ? `${e.target.value}T23:59:59Z` : undefined })}
                      />
                    </div>
                  </div>
                  {updateMutation.error && (
                    <div className="text-sm text-red-500">{updateMutation.error.message}</div>
                  )}
                </div>
              ) : (
                <div className="space-y-3 text-sm">
                  {task.description && (
                    <div>
                      <Label className="text-muted-foreground text-xs">Description</Label>
                      <p className="whitespace-pre-wrap">{task.description}</p>
                    </div>
                  )}
                  <div className="grid grid-cols-3 gap-4">
                    <div>
                      <Label className="text-muted-foreground text-xs">Type</Label>
                      <p>{typeLabels[task.task_type] || task.task_type}</p>
                    </div>
                    <div>
                      <Label className="text-muted-foreground text-xs">Priority</Label>
                      <Badge variant={priorityVariants[task.priority] || 'secondary'} className="mt-1">
                        {formatStatus(task.priority)}
                      </Badge>
                    </div>
                    <div>
                      <Label className="text-muted-foreground text-xs">Status</Label>
                      <Badge variant={isOverdue ? 'destructive' : statusVariants[task.status] || 'secondary'} className="mt-1">
                        {isOverdue ? 'Overdue' : formatStatus(task.status)}
                      </Badge>
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label className="text-muted-foreground text-xs">Assignee</Label>
                      <p className="flex items-center gap-1">
                        <User className="h-3 w-3" />
                        {task.assignee_name || 'Unassigned'}
                      </p>
                    </div>
                    <div>
                      <Label className="text-muted-foreground text-xs">Due Date</Label>
                      <p className={`flex items-center gap-1 ${isOverdue ? 'text-red-500' : ''}`}>
                        <Calendar className="h-3 w-3" />
                        {task.due_at ? formatDate(task.due_at) : 'Not set'}
                      </p>
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label className="text-muted-foreground text-xs">Created</Label>
                      <p className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {formatDateTime(task.created_at)}
                      </p>
                    </div>
                    {task.completed_at && (
                      <div>
                        <Label className="text-muted-foreground text-xs">Completed</Label>
                        <p className="flex items-center gap-1">
                          <CheckCircle className="h-3 w-3 text-green-500" />
                          {formatDateTime(task.completed_at)}
                        </p>
                      </div>
                    )}
                  </div>
                  {task.created_by_name && (
                    <div>
                      <Label className="text-muted-foreground text-xs">Created By</Label>
                      <p>{task.created_by_name}</p>
                    </div>
                  )}
                </div>
              )}
            </div>

            <Separator />

            {/* Comments */}
            <div className="space-y-4">
              <h3 className="font-semibold flex items-center gap-2">
                <MessageSquare className="h-4 w-4" />
                Comments
                <Badge variant="secondary">{comments?.length || 0}</Badge>
              </h3>

              {/* Add Comment */}
              <div className="flex gap-2">
                <Textarea
                  placeholder="Add a comment..."
                  value={newComment}
                  onChange={(e) => setNewComment(e.target.value)}
                  rows={2}
                  className="flex-1"
                />
                <Button
                  size="sm"
                  onClick={handleAddComment}
                  disabled={addCommentMutation.isLoading || !newComment.trim()}
                >
                  <Send className="h-4 w-4" />
                </Button>
              </div>

              {comments && comments.length > 0 ? (
                <div className="space-y-3">
                  {comments.map((c) => (
                    <div key={c.id} className="border rounded-lg p-3">
                      <div className="flex items-center justify-between mb-2">
                        <span className="font-medium text-sm">
                          {c.user_name || 'Unknown User'}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {formatDateTime(c.created_at)}
                        </span>
                      </div>
                      <p className="text-sm whitespace-pre-wrap">
                        {c.content}
                      </p>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-center py-6 border rounded-lg border-dashed">
                  <MessageSquare className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                  <p className="text-sm text-muted-foreground">
                    No comments yet. Add one above.
                  </p>
                </div>
              )}
            </div>
          </div>
        )}
      </SheetContent>
    </Sheet>
  )
}
