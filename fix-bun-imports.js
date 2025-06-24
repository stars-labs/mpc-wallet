const fs = require('fs');
const path = require('path');
const glob = require('glob');

// Find all test files
const testFiles = glob.sync('tests/**/*.test.ts');

testFiles.forEach(file => {
    console.log(`Fixing ${file}...`);
    let content = fs.readFileSync(file, 'utf8');
    
    // Fix multiple bun:test imports - combine them
    const bunImports = [];
    const jestImports = [];
    
    // Extract all bun:test imports
    const importRegex = /import\s*{([^}]+)}\s*from\s*['"]bun:test['"]\s*;?/g;
    let match;
    while ((match = importRegex.exec(content)) !== null) {
        const imports = match[1].split(',').map(s => s.trim()).filter(s => s);
        if (imports.some(i => i === 'jest')) {
            jestImports.push(...imports.filter(i => i === 'jest'));
        } else {
            bunImports.push(...imports);
        }
    }
    
    // Remove all existing bun:test imports
    content = content.replace(/import\s*{[^}]+}\s*from\s*['"]bun:test['"]\s*;?\n?/g, '');
    
    // Add combined import at the beginning
    const uniqueBunImports = [...new Set(bunImports)].filter(i => i !== 'jest');
    const uniqueJestImports = [...new Set(jestImports)];
    
    let newImports = '';
    if (uniqueBunImports.length > 0) {
        newImports += `import { ${uniqueBunImports.join(', ')} } from 'bun:test';\n`;
    }
    if (uniqueJestImports.length > 0 || content.includes('jest.')) {
        newImports += `import { jest } from 'bun:test';\n`;
    }
    
    // Find the first non-import line
    const lines = content.split('\n');
    let insertIndex = 0;
    for (let i = 0; i < lines.length; i++) {
        if (lines[i].trim() && !lines[i].startsWith('import') && !lines[i].startsWith('//')) {
            insertIndex = i;
            break;
        }
    }
    
    // Insert the new imports
    lines.splice(insertIndex, 0, newImports);
    content = lines.join('\n');
    
    // Fix any broken describe blocks
    content = content.replace(/describe\('([^']+)'\s*,\s*\(\)\s*=>\s*{\s*$/gm, "describe('$1', () => {");
    
    // Remove duplicate empty lines
    content = content.replace(/\n\n\n+/g, '\n\n');
    
    fs.writeFileSync(file, content);
});

console.log('Import fixes complete!');