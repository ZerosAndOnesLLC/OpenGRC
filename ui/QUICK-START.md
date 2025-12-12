# OpenGRC UI - Quick Start

## Getting Started (3 Steps)

### 1. Install
```bash
cd /home/mack/dev/opengrc/ui
npm install
```

### 2. Configure
```bash
cp .env.example .env.local
# Edit .env.local if needed
```

### 3. Run
```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000)

## What You'll See

A fully functional UI with:
- Dashboard with stats overview
- Complete navigation sidebar
- 14 pages ready for data:
  - Frameworks
  - Controls
  - Evidence
  - Policies
  - Risks
  - Vendors
  - Assets
  - Access Reviews
  - Audits
  - Tasks
  - Reports
  - Integrations
  - Settings
- Dark/light mode toggle
- Professional design

## To Build for Production

```bash
npm run build
```

Output: `out/` directory (ready for S3)

## Project Structure at a Glance

```
src/
├── app/           # All pages (Next.js App Router)
├── components/    # Reusable UI components
├── contexts/      # React contexts (auth, etc.)
├── lib/           # Utilities (API client, helpers)
└── styles/        # Global CSS
```

## Key Files

- `src/lib/api-client.ts` - API client for backend calls
- `src/contexts/auth-context.tsx` - Authentication state
- `src/components/sidebar.tsx` - Navigation
- `src/app/layout.tsx` - Root layout
- `.env.local` - Environment variables

## Common Commands

```bash
npm run dev      # Start dev server
npm run build    # Build for production
npm run lint     # Run linter
```

## Adding Features

### Example: Fetch Data

```typescript
import { apiClient } from '@/lib/api-client'

// In your component
const fetchData = async () => {
  const data = await apiClient.get('/controls')
  console.log(data)
}
```

### Example: Use Auth

```typescript
import { useAuth } from '@/contexts/auth-context'

// In your component
const { user, isAuthenticated } = useAuth()
```

## Next Steps

1. ✅ You're here - UI is running
2. Build the Rust API (see `/home/mack/dev/opengrc/api`)
3. Connect UI to API by updating `.env.local`
4. Start implementing features!

## Documentation

- Full docs: `README.md`
- Setup guide: `SETUP.md`
- Summary: `/home/mack/dev/opengrc/UI-SCAFFOLDING-SUMMARY.md`

## Need Help?

Check `README.md` for:
- Troubleshooting
- API integration guide
- Component usage
- Deployment instructions
