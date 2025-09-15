#!/usr/bin/env -S vhs

Output "demo.gif"

Set FontSize 40
Set Width 1400
Set Height 900
Set TypingSpeed 75ms
Set WindowBar Colorful
Set Margin 20
Set Theme "Monokai Pro"

Hide 
Sleep 0.5s 

Type "export PS1='$ '" Sleep 50ms Enter
Type "export GTK_DEBUG=''" Sleep 50ms Enter
Type "clear" Sleep 50ms Enter

# Title screen
Type "clear; figlet -f slant 'dotenvk' "
Enter
Show
Sleep 100ms
Type " A CLI tool for managing .env files from Durable Programming. Written with 🦀. '"
Enter
Sleep 100ms
Type "clear"
Enter

# Show help first
Type "# Let's see what dotenvk can do"
Enter
Sleep 0.5s
Type "dotenvk --help"
Enter
Sleep 100ms
Type "clear"
Enter

# Setting up a new project
Type "# Setting up a new project"
Enter
Sleep 0.5s
Type "mkdir my-app && cd my-app"
Enter
Sleep 0.5s
Type "# 📁 Creating initial .env file..."
Enter
Sleep 0.5s

Type "echo '# Database Configuration' > .env"
Enter
Sleep 100ms
Type "echo 'DATABASE_URL=postgres://localhost/mydb' >> .env"
Enter
Sleep 100ms
Type "echo 'DATABASE_POOL_SIZE=10' >> .env"
Enter
Sleep 100ms
Type "echo '' >> .env"
Enter
Sleep 100ms
Type "echo '# Server Configuration' >> .env"
Enter
Sleep 100ms
Type "echo 'PORT=3000' >> .env"
Enter
Sleep 100ms
Type "echo 'HOST=localhost' >> .env"
Enter
Sleep 100ms

# Show file with syntax highlighting
Type "# 📄 Current configuration:"
Enter
Type "bat .env --style=numbers,changes"
Enter
Sleep 100ms

# Add new variables with visual feedback
Type "# ➕ Adding API configuration..."
Enter
Sleep 0.5s
Type "dotenvk set API_ENDPOINT=https://api.example.com API_VERSION=v2 --comment 'API Settings'"
Enter
Sleep 100ms

Type "# ✅ Configuration updated!"
Enter
Type "bat .env --style=numbers,changes"
Enter
Sleep 100ms

# Generate secure secrets with animation
Type "# 🔐 Generating secure secrets..."
Enter
Sleep 0.5s
Type "dotenvk randomize JWT_SECRET --length 64 --alphanumeric"
Enter
Sleep 0.5s
Type "dotenvk randomize SESSION_SECRET DATABASE_PASSWORD --length 32 --symbol"
Enter
Sleep 100ms

Type "# 🎲 Secrets generated successfully!"
Enter
Type "tail -n 5 .env | bat --style=plain"
Enter
Sleep 100ms

# Interactive key listing
Type "# 🔑 All configuration keys:"
Enter
Type "dotenvk keys | column"
Enter
Sleep 100ms

# Show different export formats
Type "# 📤 Export formats available:"
Enter
Sleep 0.5s

Type "# ━━━ Shell export ━━━"
Enter
Type "dotenvk export | head -5"
Enter
Sleep 100ms

Type "# ━━━ JSON format ━━━"
Enter
Type "dotenvk export --format json | jq . | head -10"
Enter
Sleep 100ms

Type "# ━━━ YAML format ━━━"
Enter
Type "dotenvk export --format yaml | head -8"
Enter
Sleep 100ms

# Environment-specific files
Type "clear"
Enter
Type "# 🌍 Managing multiple environments"
Enter
Sleep 0.5s

Type "# Create production config"
Enter
Type "cp .env .env.production"
Enter
Type "dotenvk set NODE_ENV=production DEBUG=false --file .env.production"
Enter
Sleep 0.5s

Type "# Create staging config"
Enter
Type "cp .env .env.staging"
Enter
Type "dotenvk set NODE_ENV=staging DEBUG=true --file .env.staging"
Enter
Sleep 100ms

Type "# 📊 Comparing environments:"
Enter
Type "diff --color=always -y .env .env.production | head -10"
Enter
Sleep 100ms

# Cleanup with style
Type "cd .. && rm -rf my-app"
Enter
Sleep 0.5s

Type "clear"
Enter
Type "figlet -f small 'Thanks for watching!' | lolcat"
Enter
Hide
Type "clear; echo ''; echo '🦀 Get dotenvk at: github.com/durable programming/dotenvk' | lolcat; echo '⭐ Don'\''t forget to star the repo!' | lolcat"
Enter
Show
Sleep 4s

Hide
Sleep 0.5s
