# OpenGRC UI

Modern, responsive web interface for OpenGRC - the open-source compliance platform.

## Tech Stack

- **Framework**: Next.js 14 with App Router
- **Language**: TypeScript with strict mode
- **Styling**: Tailwind CSS
- **Components**: shadcn/ui (Radix UI primitives)
- **Icons**: Lucide React
- **Theme**: Dark mode support with next-themes
- **Deployment**: S3 static hosting with CloudFront

## Features

- Professional, clean UI design
- Full dark mode support
- Responsive sidebar navigation
- Type-safe API client
- Authentication context for TitaniumVault integration
- Reusable components (DataTable, PageHeader, Loading states)
- All core compliance pages:
  - Dashboard with overview statistics
  - Frameworks management
  - Controls tracking
  - Evidence collection
  - Policy management
  - Risk register
  - Vendor management
  - Asset inventory
  - Access reviews
  - Audit management
  - Task tracking
  - Reports generation
  - Integrations hub
  - Settings

## Getting Started

### Prerequisites

- Node.js 18+ and npm
- OpenGRC API running (see ../api/README.md)

### Installation

1. Install dependencies:

```bash
npm install
```

2. Create environment file:

```bash
cp .env.example .env.local
```

3. Update `.env.local` with your API URL:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1
```

### Development

Run the development server:

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

The app will hot-reload as you make changes.

### Building for Production

Build the static export:

```bash
npm run build
```

This creates an optimized production build in the `out/` directory with:
- Static HTML pages
- Optimized assets
- Trailing slashes for S3 compatibility

### Running Production Build Locally

To test the production build locally:

```bash
npm run start
```

Note: Since this uses static export, you'll need a static file server. You can use:

```bash
npx serve out
```

## Project Structure

```
src/
├── app/                    # Next.js App Router pages
│   ├── layout.tsx         # Root layout with sidebar
│   ├── page.tsx           # Dashboard page
│   ├── frameworks/        # Framework management
│   ├── controls/          # Controls tracking
│   ├── evidence/          # Evidence management
│   ├── policies/          # Policy management
│   ├── risks/             # Risk register
│   ├── vendors/           # Vendor management
│   ├── assets/            # Asset inventory
│   ├── access-reviews/    # Access reviews
│   ├── audits/            # Audit management
│   ├── tasks/             # Task tracking
│   ├── reports/           # Report generation
│   ├── integrations/      # Integration hub
│   └── settings/          # Settings
├── components/            # Reusable components
│   ├── ui/               # shadcn/ui components
│   ├── sidebar.tsx       # Navigation sidebar
│   ├── page-header.tsx   # Page header component
│   ├── data-table.tsx    # Generic data table
│   ├── loading.tsx       # Loading states
│   ├── theme-provider.tsx # Theme context
│   └── theme-toggle.tsx  # Dark mode toggle
├── contexts/             # React contexts
│   └── auth-context.tsx  # Authentication state
├── lib/                  # Utilities
│   ├── api-client.ts     # API client
│   └── utils.ts          # Helper functions
└── styles/               # Global styles
    └── globals.css       # Tailwind + custom CSS
```

## API Integration

The UI connects to the OpenGRC API via the API client in `src/lib/api-client.ts`.

### Authentication

Authentication is handled through TitaniumVault. The auth context (`src/contexts/auth-context.tsx`) manages:
- Login/logout
- Token storage
- User session

### Making API Calls

```typescript
import { apiClient } from '@/lib/api-client'

// GET request
const data = await apiClient.get<ResponseType>('/endpoint')

// POST request
const result = await apiClient.post<ResponseType>('/endpoint', { data })

// Upload file
const uploaded = await apiClient.upload<ResponseType>('/upload', file)
```

## Customization

### Theme Colors

Edit `src/styles/globals.css` to customize the color scheme:

```css
:root {
  --primary: 221.2 83.2% 53.3%;
  --secondary: 210 40% 96.1%;
  /* ... */
}
```

### Navigation

Edit `src/components/sidebar.tsx` to modify navigation items.

### Components

All UI components are in `src/components/ui/` and can be customized by editing the component files.

## Deployment

### S3 + CloudFront

1. Build the static export:

```bash
npm run build
```

2. Upload the `out/` directory to S3:

```bash
aws s3 sync out/ s3://your-bucket-name/ --profile prod
```

3. CloudFront will serve the static files with:
   - Trailing slash redirects (configured in next.config.js)
   - Image optimization disabled (for static export)

### Environment Variables

For production, set environment variables in your deployment pipeline:

- `NEXT_PUBLIC_API_URL`: Production API URL
- `NEXT_PUBLIC_TV_API_URL`: TitaniumVault API URL (if using)

## Development Guidelines

### Code Style

- Use TypeScript strict mode
- Follow ESLint rules (no ignores)
- Use functional components with hooks
- Prefer server components when possible

### Components

- Keep components focused and reusable
- Use shadcn/ui components as base
- Extract common patterns into shared components
- Document complex components with comments

### Performance

- Use Next.js Image component when needed (currently disabled for static export)
- Lazy load heavy components
- Memoize expensive computations
- Optimize bundle size

## Troubleshooting

### Build Errors

If you encounter build errors:

1. Clear Next.js cache: `rm -rf .next`
2. Delete node_modules: `rm -rf node_modules`
3. Reinstall: `npm install`
4. Rebuild: `npm run build`

### Type Errors

Ensure TypeScript is up to date:

```bash
npm install -D typescript@latest
```

### Styling Issues

If Tailwind styles aren't working:

1. Check `tailwind.config.ts` paths include your files
2. Verify `globals.css` is imported in root layout
3. Clear browser cache

## Contributing

See the main project README for contribution guidelines.

## License

See LICENSE in the root directory.
