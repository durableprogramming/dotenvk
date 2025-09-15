#!/usr/bin/env -S vhs

Output "quickdemo.gif"

Set FontSize 18
Set Width 1200
Set Height 600
Set TypingSpeed 120ms
Set WindowBar Colorful
Set Margin 1

Hide 
Sleep 1s 

Type "export PS1='$ '" Sleep 50ms Enter
Type "export GTK_DEBUG=''" Sleep 50ms Enter

Type "clear" Sleep 50ms Enter
Show

# Quick dotenvk Demo
Type "# Quick dotenvk demo"
Enter
Sleep 1s

# Create .env file with dotenvk
Type "dotenvk set DATABASE_URL=postgres://localhost/mydb PORT=3000"
Enter
Sleep 2s

# Generate secure passwords
Type "dotenvk randomize JWT_SECRET --length 32 --symbol"
Enter
Sleep 2s

# Show the file
Type "cat .env"
Enter
Sleep 2s

# Export as bash
Type "dotenvk export"
Enter
Sleep 2s

# Clean up
Type "rm .env"
Enter
Sleep 1s

Hide
Sleep 1s