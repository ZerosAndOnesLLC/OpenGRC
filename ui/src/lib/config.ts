/**
 * OpenGRC Application Configuration
 */

interface AppConfig {
  api: {
    baseUrl: string
  }
  auth: {
    tvApiUrl: string
    ssoLoginUrl: string
    clientId: string
  }
  app: {
    name: string
    version: string
  }
}

const getApiUrl = () => {
  if (process.env.NEXT_PUBLIC_API_URL) {
    return process.env.NEXT_PUBLIC_API_URL
  }
  return process.env.NODE_ENV === "development"
    ? "http://localhost:8080/api/v1"
    : "https://opengrc-api.your-domain.com/api/v1"
}

const getTVApiUrl = () => {
  if (process.env.NEXT_PUBLIC_TV_API_URL) {
    return process.env.NEXT_PUBLIC_TV_API_URL
  }
  return "https://api.titanium-vault.com"
}

const getSSOLoginUrl = () => {
  if (process.env.NEXT_PUBLIC_SSO_LOGIN_URL) {
    return process.env.NEXT_PUBLIC_SSO_LOGIN_URL
  }
  return "https://identity.zerosandones.us"
}

const getClientId = () => {
  if (process.env.NEXT_PUBLIC_TV_CLIENT_ID) {
    return process.env.NEXT_PUBLIC_TV_CLIENT_ID
  }
  return "opengrc"
}

const config: AppConfig = {
  api: {
    baseUrl: getApiUrl(),
  },
  auth: {
    tvApiUrl: getTVApiUrl(),
    ssoLoginUrl: getSSOLoginUrl(),
    clientId: getClientId(),
  },
  app: {
    name: "OpenGRC",
    version: "0.1.0",
  },
}

export default config
