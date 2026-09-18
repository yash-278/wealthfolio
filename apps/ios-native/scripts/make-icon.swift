// Draws the neutral app icon: swift scripts/make-icon.swift <output.png>
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

let size = 1024
let space = CGColorSpaceCreateDeviceRGB()
// No alpha channel: App Store Connect rejects transparent icons.
let context = CGContext(data: nil, width: size, height: size, bitsPerComponent: 8, bytesPerRow: 0, space: space,
                        bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue)!
let colors = [CGColor(red: 0.16, green: 0.46, blue: 0.44, alpha: 1), CGColor(red: 0.067, green: 0.106, blue: 0.125, alpha: 1)] as CFArray
let gradient = CGGradient(colorsSpace: space, colors: colors, locations: [0, 1])!
context.drawLinearGradient(gradient, start: CGPoint(x: 0, y: size), end: CGPoint(x: size, y: 0), options: [])

let points: [(CGFloat, CGFloat)] = [(212, 360), (400, 520), (560, 430), (812, 690)]
let line = CGMutablePath()
line.move(to: CGPoint(x: points[0].0, y: points[0].1))
points.dropFirst().forEach { line.addLine(to: CGPoint(x: $0.0, y: $0.1)) }
let area = line.mutableCopy()!
area.addLine(to: CGPoint(x: 812, y: 250)); area.addLine(to: CGPoint(x: 212, y: 250)); area.closeSubpath()
context.addPath(area); context.setFillColor(CGColor(red: 0.62, green: 0.86, blue: 0.83, alpha: 0.18)); context.fillPath()
context.addPath(line); context.setStrokeColor(CGColor(red: 0.62, green: 0.86, blue: 0.83, alpha: 1))
context.setLineWidth(56); context.setLineCap(.round); context.setLineJoin(.round); context.strokePath()
context.setFillColor(CGColor(red: 1, green: 1, blue: 1, alpha: 1))
context.fillEllipse(in: CGRect(x: 812 - 50, y: 690 - 50, width: 100, height: 100))

let url = URL(fileURLWithPath: CommandLine.arguments[1])
let destination = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(destination, context.makeImage()!, nil)
guard CGImageDestinationFinalize(destination) else { fatalError("Could not write icon") }
