"use client"

import { useState, useEffect, useCallback, useRef } from "react"
import { useRouter } from "next/navigation"
import { Search, X, Loader2, FileText, Shield, AlertTriangle, Building2, Archive, Layers, Box } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { apiClient } from "@/lib/api-client"

interface SearchResult {
  id: string
  entity_id: string
  type: string
  code: string | null
  title: string
  description: string | null
  category: string | null
  status: string | null
  path: string
}

interface SearchResponse {
  results: SearchResult[]
  total: number
  query: string
  processing_time_ms: number
}

const typeIcons: Record<string, React.ReactNode> = {
  control: <Shield className="h-4 w-4" />,
  risk: <AlertTriangle className="h-4 w-4" />,
  policy: <FileText className="h-4 w-4" />,
  evidence: <Archive className="h-4 w-4" />,
  vendor: <Building2 className="h-4 w-4" />,
  framework: <Layers className="h-4 w-4" />,
  asset: <Box className="h-4 w-4" />,
}

const typeColors: Record<string, string> = {
  control: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  risk: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
  policy: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  evidence: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  vendor: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
  framework: "bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-200",
  asset: "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200",
}

export function GlobalSearch() {
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState("")
  const [results, setResults] = useState<SearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [searchEnabled, setSearchEnabled] = useState(true)
  const inputRef = useRef<HTMLInputElement>(null)
  const router = useRouter()

  // Check if search is enabled
  useEffect(() => {
    const checkSearchStatus = async () => {
      try {
        const status = await apiClient.get<{ enabled: boolean }>("/search/status")
        setSearchEnabled(status.enabled)
      } catch {
        setSearchEnabled(false)
      }
    }
    checkSearchStatus()
  }, [])

  // Keyboard shortcut to open search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault()
        setOpen(true)
      }
    }
    document.addEventListener("keydown", handleKeyDown)
    return () => document.removeEventListener("keydown", handleKeyDown)
  }, [])

  // Focus input when dialog opens
  useEffect(() => {
    if (open && inputRef.current) {
      setTimeout(() => inputRef.current?.focus(), 0)
    }
  }, [open])

  // Debounced search
  useEffect(() => {
    if (!query.trim()) {
      setResults([])
      return
    }

    const timer = setTimeout(async () => {
      setLoading(true)
      try {
        const response = await apiClient.get<SearchResponse>(
          `/search?q=${encodeURIComponent(query)}&limit=10`
        )
        setResults(response.results)
        setSelectedIndex(0)
      } catch (error) {
        console.error("Search failed:", error)
        setResults([])
      } finally {
        setLoading(false)
      }
    }, 300)

    return () => clearTimeout(timer)
  }, [query])

  const handleSelect = useCallback((result: SearchResult) => {
    setOpen(false)
    setQuery("")
    setResults([])
    router.push(result.path)
  }, [router])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault()
      setSelectedIndex((i) => Math.min(i + 1, results.length - 1))
    } else if (e.key === "ArrowUp") {
      e.preventDefault()
      setSelectedIndex((i) => Math.max(i - 1, 0))
    } else if (e.key === "Enter" && results[selectedIndex]) {
      e.preventDefault()
      handleSelect(results[selectedIndex])
    } else if (e.key === "Escape") {
      setOpen(false)
    }
  }, [results, selectedIndex, handleSelect])

  if (!searchEnabled) {
    return null
  }

  return (
    <>
      <Button
        variant="outline"
        className="relative h-9 w-9 p-0 xl:h-10 xl:w-60 xl:justify-start xl:px-3 xl:py-2"
        onClick={() => setOpen(true)}
      >
        <Search className="h-4 w-4 xl:mr-2" />
        <span className="hidden xl:inline-flex">Search...</span>
        <kbd className="pointer-events-none absolute right-1.5 top-1.5 hidden h-6 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 xl:flex">
          <span className="text-xs">&#8984;</span>K
        </kbd>
      </Button>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-2xl p-0 gap-0">
          <DialogHeader className="sr-only">
            <DialogTitle>Search</DialogTitle>
          </DialogHeader>
          <div className="flex items-center border-b px-3">
            <Search className="h-4 w-4 shrink-0 opacity-50" />
            <Input
              ref={inputRef}
              placeholder="Search controls, risks, policies, vendors..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              className="flex h-12 w-full rounded-md bg-transparent py-3 text-sm outline-none border-0 focus-visible:ring-0 placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50"
            />
            {query && (
              <Button
                variant="ghost"
                size="sm"
                className="h-6 w-6 p-0"
                onClick={() => setQuery("")}
              >
                <X className="h-4 w-4" />
              </Button>
            )}
          </div>

          <div className="max-h-[400px] overflow-y-auto">
            {loading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : results.length > 0 ? (
              <div className="p-2">
                {results.map((result, index) => (
                  <button
                    key={result.id}
                    className={`w-full flex items-start gap-3 rounded-md px-3 py-2 text-left transition-colors ${
                      index === selectedIndex
                        ? "bg-accent text-accent-foreground"
                        : "hover:bg-accent/50"
                    }`}
                    onClick={() => handleSelect(result)}
                    onMouseEnter={() => setSelectedIndex(index)}
                  >
                    <div className={`mt-0.5 p-1.5 rounded ${typeColors[result.type] || "bg-gray-100"}`}>
                      {typeIcons[result.type] || <FileText className="h-4 w-4" />}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        {result.code && (
                          <span className="text-xs font-mono text-muted-foreground">
                            {result.code}
                          </span>
                        )}
                        <span className="font-medium truncate">{result.title}</span>
                      </div>
                      {result.description && (
                        <p className="text-xs text-muted-foreground truncate mt-0.5">
                          {result.description.substring(0, 100)}
                          {result.description.length > 100 ? "..." : ""}
                        </p>
                      )}
                      <div className="flex items-center gap-2 mt-1">
                        <Badge variant="outline" className="text-xs capitalize">
                          {result.type}
                        </Badge>
                        {result.category && (
                          <Badge variant="secondary" className="text-xs">
                            {result.category}
                          </Badge>
                        )}
                        {result.status && (
                          <Badge variant="secondary" className="text-xs capitalize">
                            {result.status.replace("_", " ")}
                          </Badge>
                        )}
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            ) : query.trim() ? (
              <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                <Search className="h-8 w-8 mb-2 opacity-50" />
                <p className="text-sm">No results found for &quot;{query}&quot;</p>
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                <Search className="h-8 w-8 mb-2 opacity-50" />
                <p className="text-sm">Start typing to search...</p>
                <p className="text-xs mt-1">
                  Search across controls, risks, policies, vendors, and more
                </p>
              </div>
            )}
          </div>

          <div className="flex items-center justify-between border-t px-3 py-2 text-xs text-muted-foreground">
            <div className="flex items-center gap-2">
              <kbd className="rounded border bg-muted px-1.5 py-0.5">&#8593;</kbd>
              <kbd className="rounded border bg-muted px-1.5 py-0.5">&#8595;</kbd>
              <span>to navigate</span>
            </div>
            <div className="flex items-center gap-2">
              <kbd className="rounded border bg-muted px-1.5 py-0.5">&#9166;</kbd>
              <span>to select</span>
            </div>
            <div className="flex items-center gap-2">
              <kbd className="rounded border bg-muted px-1.5 py-0.5">esc</kbd>
              <span>to close</span>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}
