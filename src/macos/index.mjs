import * as esbuild from 'esbuild';

esbuild.buildSync({
    entryPoints: ['/Users/chiwang/code/frace/src/macos/zzz.js'],
    minify: true,
});
