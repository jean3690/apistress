import sharp from 'sharp'
import { writeFileSync, readFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const iconsDir = join(__dirname, '..', 'src-tauri', 'icons')
const svg = readFileSync(join(iconsDir, 'icon.svg'))

const sizes = [
  { name: '32x32.png', size: 32 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: 'icon.png', size: 512 },
  { name: 'Square30x30Logo.png', size: 30 },
  { name: 'Square44x44Logo.png', size: 44 },
  { name: 'Square71x71Logo.png', size: 71 },
  { name: 'Square89x89Logo.png', size: 89 },
  { name: 'Square107x107Logo.png', size: 107 },
  { name: 'Square142x142Logo.png', size: 142 },
  { name: 'Square150x150Logo.png', size: 150 },
  { name: 'Square284x284Logo.png', size: 284 },
  { name: 'Square310x310Logo.png', size: 310 },
  { name: 'StoreLogo.png', size: 100 },
]

async function generate() {
  for (const { name, size } of sizes) {
    const path = join(iconsDir, name)
    await sharp(svg).resize(size, size).png().toFile(path)
    console.log(`  ${name} (${size}x${size})`)
  }

  // Generate ICO (multi-size, for Windows)
  const icoPath = join(iconsDir, 'icon.ico')
  const icoSizes = [16, 32, 48, 256]
  // sharp doesn't directly output .ico, so use the 256px PNG and png-to-ico approach
  // On Windows, just use the PNG; Tauri can accept .png renamed to .ico for dev
  // For production: copy the 256 PNG first, then note that proper .ico needs external tool
  const png256 = await sharp(svg).resize(256, 256).png().toBuffer()
  writeFileSync(icoPath, png256)
  console.log('  icon.ico (256x256 PNG — use icotool or ImageMagick for multi-res)')

  // Generate ICNS placeholder (macOS) — copy largest PNG
  const icnsPath = join(iconsDir, 'icon.icns')
  const png1024 = await sharp(svg).resize(1024, 1024).png().toBuffer()
  writeFileSync(icnsPath, png1024)
  console.log('  icon.icns (1024x1024 PNG — use iconutil on macOS)')

  console.log('Done.')
}

generate().catch(console.error)
