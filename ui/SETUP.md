# OpenGRC UI - Quick Setup Guide

## Installation Steps

### 1. Install Dependencies

```bash
cd /home/mack/dev/opengrc/ui
npm install
```

This will install all required packages including:
- Next.js 14
- React 18
- TypeScript
- Tailwind CSS
- shadcn/ui components
- Radix UI primitives
- Lucide icons
- next-themes for dark mode

### 2. Configure Environment

Create a `.env.local` file:

```bash
cp .env.example .env.local
```

Update the API URL in `.env.local`:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1
```

### 3. Run Development Server

```bash
npm run dev
```

The app will be available at [http://localhost:3000](http://localhost:3000)

### 4. Build for Production

```bash
npm run build
```

This creates a static export in the `out/` directory ready for S3 deployment.

## Project Structure

```
opengrc/ui/
├── src/
│   ├── app/                    # Next.js App Router pages
│   │   ├── layout.tsx         # Root layout with sidebar
│   │   ├── page.tsx           # Dashboard
│   │   ├── frameworks/        # Frameworks management
│   │   ├── controls/          # Controls tracking
│   │   ├── evidence/          # Evidence management
│   │   ├── policies/          # Policy management
│   │   ├── risks/             # Risk register
│   │   ├── vendors/           # Vendor management
│   │   ├── assets/            # Asset inventory
│   │   ├── access-reviews/    # Access reviews
│   │   ├── audits/            # Audit management
│   │   ├── tasks/             # Task tracking
│   │   ├── reports/           # Report generation
│   │   ├── integrations/      # Integration hub
│   │   └── settings/          # Settings
│   ├── components/
│   │   ├── ui/               # shadcn/ui base components
│   │   │   ├── button.tsx
│   │   │   ├── card.tsx
│   │   │   └── separator.tsx
│   │   ├── sidebar.tsx       # Navigation sidebar
│   │   ├── page-header.tsx   # Reusable page header
│   │   ├── data-table.tsx    # Generic data table
│   │   ├── loading.tsx       # Loading components
│   │   ├── theme-provider.tsx
│   │   └── theme-toggle.tsx
│   ├── contexts/
│   │   └── auth-context.tsx  # Auth state management
│   ├── lib/
│   │   ├── api-client.ts     # API client with auth
│   │   └── utils.ts          # Utility functions
│   └── styles/
│       └── globals.css       # Global styles + Tailwind
├── public/                    # Static assets
├── .env.example              # Environment template
├── .eslintrc.json           # ESLint config
├── .gitignore               # Git ignore
├── next.config.js           # Next.js config (static export)
├── package.json             # Dependencies
├── postcss.config.js        # PostCSS config
├── tailwind.config.ts       # Tailwind config
├── tsconfig.json            # TypeScript config
└── README.md                # Main documentation
```

## Features Implemented

### Core Infrastructure
- ✅ Next.js 14 with App Router
- ✅ TypeScript with strict mode
- ✅ Tailwind CSS with custom theme
- ✅ shadcn/ui component library
- ✅ Dark mode support
- ✅ S3-compatible static export

### Layout & Navigation
- ✅ Professional sidebar navigation
- ✅ Responsive design
- ✅ Theme toggle (dark/light mode)
- ✅ Consistent page headers

### Pages (All Placeholder Ready)
- ✅ Dashboard with stats overview
- ✅ Frameworks
- ✅ Controls
- ✅ Evidence
- ✅ Policies
- ✅ Risks
- ✅ Vendors
- ✅ Assets
- ✅ Access Reviews
- ✅ Audits
- ✅ Tasks
- ✅ Reports
- ✅ Integrations
- ✅ Settings

### Reusable Components
- ✅ PageHeader - Consistent page titles with actions
- ✅ DataTable - Generic table for list views
- ✅ Loading - Loading states
- ✅ Button, Card, Separator - Base UI components

### Utilities
- ✅ API Client - Type-safe HTTP client with auth
- ✅ Auth Context - Authentication state management
- ✅ Theme Provider - Dark mode support

## Next Steps

### To Make It Functional

1. **Connect to API**: The UI is ready to connect to the Rust API once it's built
2. **Implement Forms**: Add forms for creating/editing entities
3. **Add Validation**: Implement form validation
4. **Real Data**: Replace placeholder data with API calls
5. **File Upload**: Implement evidence file upload
6. **Search**: Add search functionality
7. **Filters**: Add filtering to data tables
8. **Pagination**: Add pagination to lists
9. **Notifications**: Implement toast notifications
10. **Error Handling**: Add comprehensive error handling

### Deployment

The project is configured for S3 static hosting:

```bash
# Build
npm run build

# Deploy to S3 (using AWS CLI)
aws s3 sync out/ s3://your-bucket-name/ --profile prod

# Or use the Terraform setup in ~/dev/terraform/prod/us-east-1
```

## Development Commands

```bash
# Install dependencies
npm install

# Start dev server (with hot reload)
npm run dev

# Lint code
npm run lint

# Build for production
npm run build

# Run production build locally
npx serve out
```

## Troubleshooting

### "Module not found" errors
```bash
rm -rf node_modules package-lock.json
npm install
```

### Build fails
```bash
rm -rf .next
npm run build
```

### Types errors
```bash
npm install -D typescript@latest
```

## Design System

### Colors
The app uses a professional blue theme optimized for compliance software:
- Primary: Blue (#3b82f6 in light, #60a5fa in dark)
- Muted: Gray tones for secondary content
- Destructive: Red for warnings/errors
- Professional, easy on the eyes for long sessions

### Typography
- Inter font family (clean, modern)
- Consistent sizing scale
- Clear hierarchy

### Components
- Rounded corners (8px default)
- Subtle shadows
- Smooth transitions
- Hover states on interactive elements

## Contributing

When adding new features:
1. Create components in `src/components/`
2. Add pages in `src/app/`
3. Use the API client for data fetching
4. Follow TypeScript strict mode
5. Keep ESLint warnings at zero
6. Test in both light and dark mode

## Support

See the main OpenGRC documentation for more details.
