import path from "path";
import {existsSync, mkdirSync, writeFile} from "fs";
import {promisify} from "util";
import fs from "fs";
import {convertPDFtoPNG} from "./render";

// The folder that contains the svg test files.
const svgFolderPath = "svgs";
// The folder that contains the reference images for the svg test files.
const referencesFolderPath = "references";
// A temporary folder where pdfs generated by svg2pdf will be stored.
const pdfsFolderPath = "pdfs";
// The folder that stores the difference between images if a test fails.
const diffsFolderPath = "diffs";
// Path to the svg2pdf binary once it has been built.
const pdf2svgBinaryPath = path.join("..", "target", "release", "svg2pdf");


const exec = promisify(require('child_process').exec);

const SKIPPED_FILES = [
    // These files crash svg2pdf so we always skip them.
    'resvg/structure/svg/zero-size.svg',
    'resvg/structure/svg/not-UTF-8-encoding.svg',
    'resvg/structure/svg/negative-size.svg',

    // These files don't work correctly in resvg (https://razrfalcon.github.io/resvg-test-suite/svg-support-table.html)
    // or are marked as "undefined behavior", so we skip them as well
    'resvg/shapes/rect/cap-values.svg',
    'resvg/shapes/rect/ch-values.svg',
    'resvg/shapes/rect/ic-values.svg',
    'resvg/shapes/rect/lh-values.svg',
    'resvg/shapes/rect/q-values.svg',
    'resvg/shapes/rect/rem-values.svg',
    'resvg/shapes/rect/rlh-values.svg',
    'resvg/shapes/rect/vi-and-vb-values.svg',
    'resvg/shapes/rect/vmin-and-vmax-values.svg',
    'resvg/shapes/rect/vw-and-vh-values.svg',

    'resvg/structure/image/float-size.svg',
    'resvg/structure/image/no-height-on-svg.svg',
    'resvg/structure/image/no-width-and-height-on-svg.svg',
    'resvg/structure/image/no-width-on-svg.svg',
    'resvg/structure/image/url-to-png.svg',
    'resvg/structure/image/url-to-svg.svg',

    'resvg/structure/style/external-CSS.svg',
    'resvg/structure/style/important.svg',

    'resvg/structure/svg/funcIRI-parsing.svg',
    'resvg/structure/svg/invalid-id-attribute-1.svg',
    'resvg/structure/svg/invalid-id-attribute-2.svg',
    'resvg/structure/svg/not-UTF-8-encoding.svg',
    'resvg/structure/svg/xlink-to-an-external-file.svg',

    'resvg/painting/fill/#RGBA.svg',
    'resvg/painting/fill/#RRGGBBAA.svg',
    'resvg/painting/fill/icc-color.svg',
    'resvg/painting/fill/rgb-int-int-int.svg',
    'resvg/painting/fill/rgba-0-127-0-50percent.svg',
    'resvg/painting/fill/valid-FuncIRI-with-a-fallback-ICC-color.svg',

    'resvg/painting/marker/on-ArcTo.svg',
    'resvg/painting/marker/target-with-subpaths-2.svg',
    'resvg/painting/marker/with-viewBox-1.svg',

    'resvg/painting/paint-order/fill-markers-stroke.svg',
    'resvg/painting/paint-order/stroke-markers.svg',

    'resvg/painting/stroke-dasharray/negative-sum.svg',
    'resvg/painting/stroke-dasharray/negative-values.svg',

    'resvg/painting/stroke-linejoin/arcs.svg',
    'resvg/painting/stroke-linejoin/miter-clip.svg',

    'resvg/painting/stroke-width/negative.svg',

    'resvg/masking/clip/simple-case.svg',
    'resvg/masking/clipPath/on-the-root-svg-without-size.svg',

    'resvg/masking/mask/color-interpolation=linearRGB.svg',
    'resvg/masking/mask/recursive-on-child.svg',

    'resvg/paint-servers/linearGradient/invalid-gradientTransform.svg',
    'resvg/paint-servers/pattern/invalid-patternTransform.svg',
    'resvg/paint-servers/pattern/overflow=visible.svg',

    'resvg/paint-servers/radialGradient/fr=-1.svg',
    'resvg/paint-servers/radialGradient/fr=0.2.svg',
    'resvg/paint-servers/radialGradient/fr=0.5.svg',
    'resvg/paint-servers/radialGradient/fr=0.7.svg',
    'resvg/paint-servers/radialGradient/invalid-gradientTransform.svg',
    'resvg/paint-servers/radialGradient/invalid-gradientUnits.svg',
    'resvg/paint-servers/radialGradient/negative-r.svg',

    // These files contain text which currently doesn't work well with the CI,
    // so we skip them for now.
    'resvg/structure/systemLanguage/on-tspan.svg',
    'resvg/structure/svg/mixed-namespaces.svg',
    'resvg/structure/a/on-tspan.svg',
    'resvg/structure/a/inside-tspan.svg',
    'resvg/painting/visibility/hidden-on-tspan.svg',
    'resvg/painting/visibility/collapse-on-tspan.svg',
    'resvg/painting/stroke-opacity/on-text.svg',
    'resvg/painting/stroke/pattern-on-text.svg',
    'resvg/painting/marker/with-a-text-child.svg',
    'resvg/painting/fill-opacity/on-text.svg',
    'resvg/painting/fill/pattern-on-text.svg',
    'resvg/painting/display/none-on-tspan-1.svg',
    'resvg/painting/display/none-on-tref.svg',
]

// Converts the svg from the input path to a pdf and saves it in the output path.
async function generateAndWritePDF(inputPath: string, outputPath: string) {
    let outputFolderPath = path.dirname(outputPath);
    let command = pdf2svgBinaryPath + ' ' + inputPath + ' ' + outputPath;

    if (!existsSync(outputFolderPath)) {
        mkdirSync(outputFolderPath, {recursive: true});
    }

    try {
        await exec(command);
    } catch (e: any) {
        throw new Error("error while generating the pdf: " + e.message);
    }
}

// Converts the pdf from the input path to a png and returns it as a buffer.
async function generatePNG(inputFilePath: string): Promise<Uint8Array> {
    return await convertPDFtoPNG(new Uint8Array(fs.readFileSync(inputFilePath)));
}

// Converts the pdf from the input path to a png and saves it in the output path.
async function generateAndWritePNG(inputFilePath: string, outputFilePath: string) {
    let pdfImage = await generatePNG(inputFilePath);

    let outputFolderPath = path.dirname(outputFilePath);

    if (!existsSync(outputFolderPath)) {
        mkdirSync(outputFolderPath, {recursive: true});
    }

    await writeFile(outputFilePath, pdfImage, function (error) {
        if (error) {
            throw new Error("unable to write image to file system: " + error)
        }
    });
}

async function writeDiffImage(diffImage: Buffer, actualImage: Buffer, referenceImage: Buffer, outputPath: string) {
    let outputFolderPath = path.dirname(outputPath);

    if (!existsSync(outputFolderPath)) {
        mkdirSync(outputFolderPath, {recursive: true});
    }

    let generateOutputPath = (nameExtension: string): string => {
        return path.join(path.dirname(outputPath),
            path.basename(outputPath, path.extname(outputPath)) + "-" + nameExtension + path.extname(outputPath));
    }

    const diffOutputPath = generateOutputPath("diff");
    const actualOutputPath = generateOutputPath("actual");
    const referenceOutputPath = generateOutputPath("reference");

    await writeFile(diffOutputPath, diffImage, function (error) {
        if (error) {
            throw new Error("unable to write diff image to file system: " + error)
        }
    });

    await writeFile(actualOutputPath, actualImage, function (error) {
        if (error) {
            throw new Error("unable to write actual image to file system: " + error)
        }
    });

    await writeFile(referenceOutputPath, referenceImage, function (error) {
        if (error) {
            throw new Error("unable to write reference image to file system: " + error)
        }
    });
}

function generateFullPath(parentFolder: string, filePath: string, extension: string) {
    return path.join(parentFolder, path.dirname(filePath),
        path.basename(filePath, path.extname(filePath)) + "." + extension);
}

// Takes a path like 'resvg/shapes/lines/no-coordinates.svg' and turns it into
// 'svgs/resvg/shapes/lines/no-coordinates.svg'
function generateSVGPath(filePath: string) {
    return generateFullPath(svgFolderPath, filePath, "svg");
}

// Takes a path like 'resvg/shapes/lines/no-coordinates.svg' and turns it into
// 'references/resvg/shapes/lines/no-coordinates.png'
function generateReferencePath(filePath: string) {
    return generateFullPath(referencesFolderPath, filePath, "png");
}

// Takes a path like 'resvg/shapes/lines/no-coordinates.svg' and turns it into
// 'pdfs/resvg/shapes/lines/no-coordinates.pdf'
function generatePDFPath(filePath: string) {
    return generateFullPath(pdfsFolderPath, filePath, "pdf");
}

// Takes a path like 'resvg/shapes/lines/no-coordinates.svg' and turns it into
// 'diffs/resvg/shapes/lines/no-coordinates.png'
function generateDiffsPath(filePath: string) {
    return generateFullPath(diffsFolderPath, filePath, "png");
}

function replaceExtension(replacePath: string, extension: string) {
    return path.join(path.dirname(replacePath),
    path.basename(replacePath, path.extname(replacePath)) + "." + extension);
}

function clearPDFs() {
    if (existsSync(pdfsFolderPath)) {
        fs.rmSync(pdfsFolderPath, { recursive: true});
    }
}

function clearDiffs() {
    if (existsSync(diffsFolderPath)) {
        fs.rmSync(diffsFolderPath, { recursive: true});
    }
}

export {
    svgFolderPath, referencesFolderPath, pdfsFolderPath, pdf2svgBinaryPath, generateAndWritePNG,
    generateAndWritePDF, replaceExtension, generatePNG, generateSVGPath,
    generatePDFPath, generateReferencePath, generateDiffsPath, clearPDFs, clearDiffs, writeDiffImage,
    SKIPPED_FILES
}